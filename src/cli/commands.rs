use clap::Subcommand;
use anyhow::Result;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 启动交互式会话
    Chat,
    /// 处理单个会话请求
    Process {
        /// 会话数据JSON文件路径
        #[arg(short, long)]
        file: String,
    },
}

pub fn handle_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Chat => {
            println!("启动交互式聊天模式");
            // TODO: 实现交互逻辑
            Ok(())
        }
        Commands::Process { file } => {
            println!("处理文件: {}", file);
            // TODO: 实现文件处理逻辑
            Ok(())
        }
    }
}