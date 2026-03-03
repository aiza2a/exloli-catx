async fn cmd_best(
    bot: Bot,
    msg: Message,
    args: String,
    cfg: Config,
    scheduler: Scheduler,
) -> Result<()> {
    let parts: Vec<&str> = args.split_whitespace().collect();
    
    if parts.len() != 2 {
        reply_to!(
            bot, 
            msg, 
            "<b>使用說明：</b>\n查詢指定時間範圍內的熱門本子。\n\n<b>格式：</b>\n<code>/best [天數1] [天數2]</code>\n\n<b>示例：</b>\n<code>/best 30 0</code> (查詢最近30天)\n<code>/best 30 60</code> (查詢上個月)"
        ).await?;
        return Ok(());
    }

    let day1: i32 = match parts[0].parse() {
        Ok(v) => v,
        Err(_) => { reply_to!(bot, msg, "❌ 第一個參數必須是數字").await?; return Ok(()); }
    };
    let day2: i32 = match parts[1].parse() {
        Ok(v) => v,
        Err(_) => { reply_to!(bot, msg, "❌ 第二個參數必須是數字").await?; return Ok(()); }
    };

    // 此處已刪除大小判斷，直接傳給 utils 自動排序適配
    info!("{}: /best {} {}", msg.from().unwrap().id, day1, day2);
    
    let text = cmd_best_text(day1, day2, 0, cfg.telegram.channel_id).await?;
    let keyboard = cmd_best_keyboard(day1, day2, 0);
    let reply =
        reply_to!(bot, msg, text).reply_markup(keyboard).disable_web_page_preview(true).await?;
        
    if !msg.chat.is_private() {
        scheduler.delete_msg(msg.chat.id, msg.id, 120);
        scheduler.delete_msg(msg.chat.id, reply.id, 120);
    }
    Ok(())
}
