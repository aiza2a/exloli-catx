use anyhow::{anyhow, Context, Result};
use rand::prelude::*;
use reqwest::Url;
use std::str::FromStr;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::InputFile;
use teloxide::utils::html::escape;
use tracing::info;

use crate::bot::command::{AdminCommand, PublicCommand};
use crate::bot::handlers::{
    cmd_best_keyboard, cmd_best_text, cmd_challenge_keyboard, gallery_preview_url,
};
use crate::bot::scheduler::Scheduler;
use crate::bot::utils::{ChallengeLocker, ChallengeProvider};
use crate::bot::Bot;
use crate::config::Config;
use crate::database::{GalleryEntity, ImageEntity, MessageEntity, PollEntity};
use crate::ehentai::EhGalleryUrl;
use crate::tags::EhTagTransDB;
use crate::uploader::ExloliUploader;
use crate::{reply_to, try_with_reply};

const INDENT: &str = "\u{2063}\u{3000}";

pub fn public_command_handler(
    _config: Config,
) -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription> {
    teloxide::filter_command::<PublicCommand, _>()
        .branch(case![PublicCommand::Query(args)].endpoint(cmd_query))
        .branch(case![PublicCommand::Ping].endpoint(cmd_ping))
        .branch(case![PublicCommand::Update(url)].endpoint(cmd_update))
        .branch(case![PublicCommand::Best(args)].endpoint(cmd_best))
        .branch(case![PublicCommand::Challenge].endpoint(cmd_challenge))
        .branch(case![PublicCommand::Upload(args)].endpoint(cmd_upload))
        .branch(case![PublicCommand::Random].endpoint(cmd_random))
        .branch(case![PublicCommand::Stats].endpoint(cmd_stats))
        .branch(case![PublicCommand::Help].endpoint(cmd_help))
}

async fn cmd_help(bot: Bot, msg: Message) -> Result<()> {
    let text = format!(
        "<b>🕹 公共指令菜單</b>\n\n\
        <code>/random</code>\n{i}隨機抽取藏書\n\n\
        <code>/stats</code>\n{i}查看統計信息\n\n\
        <code>/query [url]</code>\n{i}查詢收錄狀態\n\n\
        <code>/best [最近] [最遠]</code>\n{i}熱門排行 (如 /best 30 0)\n\n\
        <code>/help</code>\n{i}顯示本菜單",
        i = INDENT
    );
    reply_to!(bot, msg, text).await?;
    Ok(())
}

async fn cmd_upload(bot: Bot, msg: Message, uploader: ExloliUploader, url_text: String) -> Result<()> {
    if url_text.trim().is_empty() {
        reply_to!(bot, msg, format!("<b>ℹ️ 使用提示</b>\n{i}請附上鏈接，例如：\n{i}<code>/upload https://...</code>", i = INDENT)).await?;
        return Ok(());
    }
    let gallery = match EhGalleryUrl::from_str(&url_text) {
        Ok(v) => v,
        Err(_) => { reply_to!(bot, msg, format!("<b>🚫 錯誤</b>\n{i}鏈接格式不正確。", i = INDENT)).await?; return Ok(()); }
    };
    if GalleryEntity::get(gallery.id()).await?.is_none() {
        reply_to!(bot, msg, format!("<b>⚠️ 權限不足</b>\n{i}非管理員僅能上傳已收錄過的畫廊。", i = INDENT)).await?;
    } else {
        try_with_reply!(bot, msg, uploader.try_upload(&gallery, true).await);
    }
    Ok(())
}

async fn cmd_challenge(bot: Bot, msg: Message, trans: EhTagTransDB, locker: ChallengeLocker, scheduler: Scheduler, challange_provider: ChallengeProvider) -> Result<()> {
    let mut challenge = challange_provider.get_challenge().await.unwrap();
    let answer = challenge[0].clone();
    challenge.shuffle(&mut thread_rng());
    let url = if answer.url.starts_with("http") { answer.url.clone() } else { format!("https://telegra.ph{}", answer.url) };
    let id = locker.add_challenge(answer.id, answer.page, answer.artist.clone());
    let keyboard = cmd_challenge_keyboard(id, &challenge, &trans);
    let reply = bot.send_photo(msg.chat.id, InputFile::url(url.parse()?)).caption(format!("<b>🎲 猜作者遊戲</b>\n{i}圖片來自誰的本子？", i = INDENT)).reply_markup(keyboard).reply_to_message_id(msg.id).await?;
    if !msg.chat.is_private() { scheduler.delete_msg(msg.chat.id, msg.id, 120); scheduler.delete_msg(msg.chat.id, reply.id, 120); }
    Ok(())
}

async fn cmd_best(bot: Bot, msg: Message, args: String, cfg: Config, scheduler: Scheduler) -> Result<()> {
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.len() != 2 {
        reply_to!(bot, msg, format!("<b>ℹ️ 使用提示</b>\n{i}格式：<code>/best [最近天數] [最遠天數]</code>", i = INDENT)).await?;
        return Ok(());
    }
    let end: u16 = parts[0].parse().unwrap_or(0);
    let start: u16 = parts[1].parse().unwrap_or(0);
    let text = cmd_best_text(start as i32, end as i32, 0, cfg.telegram.channel_id).await?;
    let keyboard = cmd_best_keyboard(start as i32, end as i32, 0);
    let reply = reply_to!(bot, msg, text).reply_markup(keyboard).disable_web_page_preview(true).await?;
    if !msg.chat.is_private() { scheduler.delete_msg(msg.chat.id, msg.id, 120); scheduler.delete_msg(msg.chat.id, reply.id, 120); }
    Ok(())
}

async fn cmd_update(bot: Bot, msg: Message, uploader: ExloliUploader, url: String) -> Result<()> {
    let msg_id = if url.is_empty() {
        msg.reply_to_message().and_then(|m| m.forward_from_message_id()).ok_or(anyhow!("Empty"))
    } else {
        Url::parse(&url).map_err(|_| anyhow!("Err")).and_then(|u| u.path_segments().and_then(|p| p.last()).and_then(|id| id.parse::<i32>().ok()).ok_or(anyhow!("Err")))
    };
    let msg_id = match msg_id {
        Ok(id) => id,
        Err(_) => { reply_to!(bot, msg, format!("<b>ℹ️ 使用提示</b>\n{i}請附上 URL 或「回覆」一條畫廊消息。", i = INDENT)).await?; return Ok(()); }
    };
    let msg_entity = MessageEntity::get(msg_id).await?.ok_or(anyhow!("None"))?;
    let gl_entity = GalleryEntity::get(msg_entity.gallery_id).await?.ok_or(anyhow!("None"))?;
    let reply = reply_to!(bot, msg, format!("<b>⏳ 更新中</b>\n{i}正在同步元數據...", i = INDENT)).await?;
    uploader.recheck(vec![gl_entity.clone()]).await?;
    uploader.try_update(&gl_entity.url(), false).await?;
    bot.edit_message_text(msg.chat.id, reply.id, format!("<b>✅ 更新完成</b>\n{i}信息已同步。", i = INDENT)).await?;
    Ok(())
}

async fn cmd_ping(bot: Bot, msg: Message, scheduler: Scheduler) -> Result<()> {
    let reply = reply_to!(bot, msg, format!("<b>🏓 Pong!</b>\n{i}夏萊閱覽室正在運行中...", i = INDENT)).await?;
    if !msg.chat.is_private() { scheduler.delete_msg(msg.chat.id, msg.id, 120); scheduler.delete_msg(msg.chat.id, reply.id, 120); }
    Ok(())
}

async fn cmd_query(bot: Bot, msg: Message, cfg: Config, gallery: String) -> Result<()> {
    if gallery.is_empty() {
        reply_to!(bot, msg, format!("<b>ℹ️ 查詢提示</b>\n{i}請附上畫廊鏈接。", i = INDENT)).await?;
        return Ok(());
    }
    let url = match EhGalleryUrl::from_str(&gallery) {
        Ok(v) => v,
        Err(_) => { reply_to!(bot, msg, format!("<b>🚫 錯誤</b>\n{i}無效的鏈接。", i = INDENT)).await?; return Ok(()); }
    };
    match GalleryEntity::get(url.id()).await? {
        Some(g) => {
            let poll = PollEntity::get_by_gallery(g.id).await?.unwrap();
            let preview = gallery_preview_url(cfg.telegram.channel_id, g.id).await?;
            reply_to!(bot, msg, format!("<b>🔍 查詢結果</b>\n\n📄 <b>預覽：</b>{}\n🔗 <b>鏈接：</b>{}\n⭐️ <b>評分：</b>{:.2}", preview, g.url().url(), poll.score * 100.)).await?;
        }
        None => { reply_to!(bot, msg, format!("<b>😶 未收錄</b>\n{i}數據庫中找不到該畫廊。", i = INDENT)).await?; }
    }
    Ok(())
}

async fn cmd_random(bot: Bot, msg: Message, cfg: Config) -> Result<()> {
    match GalleryEntity::get_random().await? {
        Some(gallery) => {
            let preview = gallery_preview_url(cfg.telegram.channel_id, gallery.id).await?;
            reply_to!(bot, msg, format!("<b>🎲 隨機抽取結果</b>\n\n📄 <b>預覽：</b>{}\n🔗 <b>鏈接：</b>{}", preview, gallery.url().url())).await?;
        }
        None => { reply_to!(bot, msg, format!("<b>😶 空空如也</b>\n{i}數據庫是空的。", i = INDENT)).await?; }
    }
    Ok(())
}

async fn cmd_stats(bot: Bot, msg: Message) -> Result<()> {
    let gallery_count = GalleryEntity::count().await?;
    let image_count = ImageEntity::count().await?;
    let text = format!("<b>📊 數據統計</b>\n\n📚 <b>畫廊數量：</b><code>{}</code>\n🖼 <b>圖片數量：</b><code>{}</code>", gallery_count, image_count);
    reply_to!(bot, msg, text).await?;
    Ok(())
}
