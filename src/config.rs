use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use log::{info, warn};

/// ソートアルゴリズムの種類
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortAlgorithm {
    /// ファイル名でソート
    FileName,
    /// 作成日時でソート
    CreatedTime,
    /// 更新日時でソート
    ModifiedTime,
}

impl Default for SortAlgorithm {
    fn default() -> Self {
        SortAlgorithm::FileName
    }
}

/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 最小ウィンドウサイズ（幅、高さ）
    pub min_window_size: (u32, u32),
    /// 画像ファイルのソートアルゴリズム
    pub sort_algorithm: SortAlgorithm,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            min_window_size: (800, 600),
            sort_algorithm: SortAlgorithm::FileName,
        }
    }
}

impl Config {
    /// 設定ファイルを読み込む
    /// 
    /// # Arguments
    /// * `config_path` - 設定ファイルのパス
    /// 
    /// # Returns
    /// * `Result<Config>` - 設定オブジェクト
    pub fn load(config_path: &Path) -> Result<Config> {
        if config_path.exists() {
            info!("設定ファイルを読み込み中: {:?}", config_path);
            let content = fs::read_to_string(config_path)?;
            let config: Config = toml::from_str(&content)?;
            info!("設定ファイルの読み込み完了");
            Ok(config)
        } else {
            warn!("設定ファイルが見つからないため、デフォルト設定を使用します: {:?}", config_path);
            let config = Config::default();
            config.save(config_path)?;
            Ok(config)
        }
    }

    /// 設定ファイルを保存する
    /// 
    /// # Arguments
    /// * `config_path` - 設定ファイルのパス
    /// 
    /// # Returns
    /// * `Result<()>` - 成功時は Ok(())
    pub fn save(&self, config_path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        info!("設定ファイルを保存しました: {:?}", config_path);
        Ok(())
    }

    /// 実行ファイルと同じディレクトリの設定ファイルパスを取得する
    /// 
    /// # Returns
    /// * `Result<std::path::PathBuf>` - 設定ファイルのパス
    pub fn get_config_path() -> Result<std::path::PathBuf> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();
        Ok(exe_dir.join("rs_fast_image_viewer.toml"))
    }
}
