# EXLOLI-IMGX

<p align="center">
  <img src="[https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)" alt="Rust">
  <img src="[https://img.shields.io/badge/Teloxide-blue?style=for-the-badge](https://img.shields.io/badge/Teloxide-blue?style=for-the-badge)" alt="Teloxide">
  <img src="[https://img.shields.io/badge/License-MIT-green?style=for-the-badge](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)" alt="License">
</p>

基于 **Rust** 与 **Teloxide** 构建的新一代 E-Hentai / ExHentai Telegram 频道自动推送与资源管理机器人。

本系统旨在提供高度稳定且自动化的同人志资源抓取、图床中转、即时预览生成，以及丰富的 Telegram 群组与私聊交互管理功能。

---

## ✨ 核心特性

* **🚀 全自动资源流转**
  支持定时扫描 E-Hentai 资源，通过多线程并发将本地获取的图片转存至 ImgBB 图床，自动生成 Telegraph 即时预览文章并推送到目标 Telegram 频道。
* **📂 个人收藏夹系统**
  支持用户在频道、群组或私聊中一键收藏目标画廊，系统提供独立分页式的专属收藏列表 (`/fav`)，并支持在公共面板动态展示单一画廊的全网收藏热度。
* **🔍 灵活的标签检索**
  内置 E-Hentai 中英标签翻译数据库，允许用户通过单一或多重组合标签，在图库中随机检索画廊资源 (`/random`)，并支持设定单次检索上限。
* **📊 社群互动与数据统计**
  频道消息转发至讨论组后，将自动生成带有实时打分系统的投票面板。提供按时间维度计算的热度排行榜 (`/best`) 以及系统整体运行数据监控 (`/stats`)。
* **🛠️ 完备的维护干预机制**
  面向管理员提供硬/软删除机制、元数据强制同步与图文缺失自动修复工具，确保内容分发体系的可靠性。

---

## 🎮 指令参考

### 👥 公共指令
| 指令 | 描述 |
| :--- | :--- |
| `/upload <URL>` | 根据 E 站 URL 上传系统中已收录的特定画廊 |
| `/update <URL>` | 依据传入的 URL 或回复消息，同步更新指定画廊的元数据 |
| `/query <URL>` | 查验目标画廊的收录状态及详细后台评分信息 |
| `/best <天1> <天2>` | 检索指定时间跨度内的优质画廊排行榜 (示例: `/best 30 0`) |
| `/random [标签] [数量]` | 基于输入标签随机提取特定数量的画廊 (最高提取上限 10 本) |
| `/fav` | 调出并管理当前交互用户的个人收藏夹 |
| `/challenge` | 触发社群专用的画师猜谜游戏 |
| `/stats` | 获取系统总收录量、总图片量及平均页面数等统计指标 |
| `/ping` | 验证服务进程在线状态与响应延迟 |
| `/help` | 输出对应权限级别的指令帮助清单 |

### 🛡️ 管理员指令
| 指令 | 描述 |
| :--- | :--- |
| `/upload <URL>` | 强制解析并上传全新画廊 (无视标准收录限制) |
| `/delete` | 对当前回复的画廊执行**软删除**操作 (隐藏记录) |
| `/erase` | 对当前回复的画廊执行**硬删除**操作 (彻底清理数据与缺页记录) |
| `/recheck` | 检测当前画廊数据完整性并重新生成 Telegraph 预览文章 |

---

## 🚀 部署说明

生产环境推荐使用 **Docker** 容器化部署，也可通过 **Cargo** 直接编译构建。

### 方式一：通过 Docker Compose (推荐)

初始化项目目录及必要配置文件：

```bash
mkdir exloli-next && cd exloli-next

# 获取配置模板与容器编排文件
wget https://raw.githubusercontent.com/lolishinshi/exloli/master/docker-compose.yml
wget https://raw.githubusercontent.com/lolishinshi/exloli/master/config.toml.example -O config.toml

# 获取标签翻译数据库
wget https://github.com/EhTagTranslation/Database/releases/download/v6.7880.1/db.text.json

# 创建 SQLite 数据文件存储占位符
touch db.sqlite db.sqlite-shm db.sqlite-wal
```

完善 `config.toml` 配置参数后，启动服务：
```bash
docker-compose up -d
```

### 方式二：通过 Cargo 源码编译

配置 Rust 工具链并构建：

```bash
# 部署 Rust 编译环境
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 编译并安装系统包
cargo install --git https://github.com/lolishinshi/exloli-next

# 测试环境变量配置与命令行输出
exloli-next --help
```

---

## ⚙️ 配置文件指引

核心服务依赖于运行目录下的 `config.toml`。请参照以下模块进行初始化配置：

* **Telegram 凭据**：正确填写机器人 Token 以及接收推送的目标 Channel ID。
* **ImgBB 图床**：配置至少一个或多个有效的 API Keys (系统内置多 Key 轮询上传机制)。
* **E-Hentai 认证**：提供含有效登录状态的 Cookie 键值 (`igneous`, `ipb_member_id`, `ipb_pass_hash`)。
* **Telegraph**：提供具备发布文章权限的 Access Token 密钥。

---

## 💾 数据迁移

若由旧版 exloli 系统进行升级迁移，直接保留原持久化存储的 `db.sqlite` 并在新环境下挂载运行即可。系统启动时将自动识别数据结构差异并执行底层重构映射 (Migrations)。

> [!WARNING]
> **警告**：建议在任何迁移操作前，对原 SQLite 文件及其日志副本进行完整备份。
