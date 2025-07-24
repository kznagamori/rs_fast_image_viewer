#![windows_subsystem = "windows"]
use anyhow::Result;
use clap::{Arg, Command};
use log::{error, info};
use std::path::Path;
use winit::event_loop::EventLoop;

mod config;
mod image_handler;
mod viewer;

use config::Config;
use image_handler::ImageHandler;
use viewer::ImageViewer;

/// ログを初期化する
/// 
/// # Returns
/// * `Result<()>` - 成功時は Ok(())
fn init_logging() -> Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("wgpu", log::LevelFilter::Warn)
        .level_for("winit", log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

/// コマンドライン引数を解析する
/// 
/// # Returns
/// * `clap::ArgMatches` - 解析されたコマンドライン引数
fn parse_args() -> clap::ArgMatches {
    Command::new("rs_fast_image_viewer")
        .version("0.1.0")
        .about("高速画像ビューアー（WebP対応）")
        .arg(
            Arg::new("path")
                .help("画像ファイルまたはディレクトリのパス")
                .required(true)
                .index(1),
        )
        .get_matches()
}

/// アプリケーションのメイン処理
/// 
/// # Returns
/// * `Result<()>` - 成功時は Ok(())
fn run_app() -> Result<()> {
    // ログを初期化
    init_logging()?;
    info!("rs_fast_image_viewer を起動しています...");

    // コマンドライン引数を解析
    let matches = parse_args();
    let input_path = matches.get_one::<String>("path").unwrap();
    let path = Path::new(input_path);

    // 設定を読み込み
    let config_path = Config::get_config_path()?;
    let config = Config::load(&config_path)?;
    info!("設定を読み込みました: {:?}", config);

    // 画像ハンドラーを初期化
    let mut image_handler = ImageHandler::new(config.clone());

    // パスの種類に応じて処理を分岐
    if path.is_file() {
        info!("画像ファイルが指定されました: {:?}", path);
        image_handler.load_images_with_target(path)?;
    } else if path.is_dir() {
        info!("ディレクトリが指定されました: {:?}", path);
        image_handler.load_images_from_directory(path)?;
    } else {
        return Err(anyhow::anyhow!("指定されたパスが存在しません: {:?}", path));
    }

    // 画像が見つからない場合はエラー
    if image_handler.is_empty() {
        return Err(anyhow::anyhow!("対応する画像ファイルが見つかりません"));
    }

    info!("{}個の画像ファイルが見つかりました", image_handler.len());

    // イベントループを作成
    let event_loop = EventLoop::new()?;

    // 画像ビューアーを初期化
    let viewer = ImageViewer::new(config, image_handler);
    info!("画像ビューアーを初期化しました");

    // アプリケーションを実行
    viewer.run(event_loop)?;

    Ok(())
}

/// メイン関数
fn main() {
    if let Err(e) = run_app() {
        error!("アプリケーションエラー: {:?}", e);
        std::process::exit(1);
    }
}