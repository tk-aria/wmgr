use clap::{App, Arg, SubCommand};

fn build_cli() -> App<'static, 'static> {
    App::new("upkg")
        .subcommand(SubCommand::with_name("sync")
            // ここに必要な引数やオプションを追加
        )
}


