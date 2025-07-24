use anyhow::Result;
use image::DynamicImage;
use log::{debug, info, error};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use crate::config::{Config, SortAlgorithm};

/// サポートされている画像フォーマット
const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];

/// 画像ファイル情報
#[derive(Debug, Clone)]
pub struct ImageFile {
    /// ファイルパス
    pub path: PathBuf,
    /// ファイル名
    pub name: String,
    /// 作成日時
    pub created: Option<SystemTime>,
    /// 更新日時
    pub modified: Option<SystemTime>,
}

impl ImageFile {
    /// 新しい ImageFile インスタンスを作成する
    /// 
    /// # Arguments
    /// * `path` - 画像ファイルのパス
    /// 
    /// # Returns
    /// * `Result<ImageFile>` - 画像ファイル情報
    pub fn new(path: PathBuf) -> Result<Self> {
        let metadata = fs::metadata(&path)?;
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        Ok(ImageFile {
            path,
            name,
            created: metadata.created().ok(),
            modified: metadata.modified().ok(),
        })
    }
}

/// 画像ハンドラー
pub struct ImageHandler {
    /// 画像ファイルのリスト
    pub images: Vec<ImageFile>,
    /// 現在の画像インデックス
    pub current_index: usize,
    /// 設定
    config: Config,
}

impl ImageHandler {
    /// 新しい ImageHandler インスタンスを作成する
    /// 
    /// # Arguments
    /// * `config` - アプリケーション設定
    /// 
    /// # Returns
    /// * `ImageHandler` - 画像ハンドラー
    pub fn new(config: Config) -> Self {
        ImageHandler {
            images: Vec::new(),
            current_index: 0,
            config,
        }
    }

    /// ディレクトリから画像ファイルを検索する
    /// 
    /// # Arguments
    /// * `dir_path` - 検索するディレクトリのパス
    /// 
    /// # Returns
    /// * `Result<()>` - 成功時は Ok(())
    pub fn load_images_from_directory(&mut self, dir_path: &Path) -> Result<()> {
        info!("ディレクトリから画像ファイルを検索中: {:?}", dir_path);
        
        let mut image_files = Vec::new();
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && self.is_supported_format(&path) {
                match ImageFile::new(path) {
                    Ok(image_file) => {
                        debug!("画像ファイルを発見: {:?}", image_file.path);
                        image_files.push(image_file);
                    }
                    Err(e) => {
                        error!("画像ファイル情報の取得に失敗: {:?}", e);
                    }
                }
            }
        }

        self.sort_images(&mut image_files);
        self.images = image_files;
        
        info!("画像ファイルの読み込み完了: {}個", self.images.len());
        Ok(())
    }

    /// 指定された画像ファイルを含むディレクトリから画像ファイルを読み込み、指定されたファイルを表示対象にする
    /// 
    /// # Arguments
    /// * `file_path` - 指定された画像ファイルのパス
    /// 
    /// # Returns
    /// * `Result<()>` - 成功時は Ok(())
    pub fn load_images_with_target(&mut self, file_path: &Path) -> Result<()> {
        let dir_path = file_path.parent()
            .ok_or_else(|| anyhow::anyhow!("ファイルの親ディレクトリが取得できません"))?;

        self.load_images_from_directory(dir_path)?;

        // 指定されたファイルのインデックスを見つける
        for (index, image_file) in self.images.iter().enumerate() {
            if image_file.path == file_path {
                self.current_index = index;
                info!("対象画像ファイルのインデックスを設定: {}", index);
                break;
            }
        }

        Ok(())
    }

    /// 画像ファイルがサポートされているフォーマットかどうかを確認する
    /// 
    /// # Arguments
    /// * `path` - 確認するファイルのパス
    /// 
    /// # Returns
    /// * `bool` - サポートされている場合は true
    fn is_supported_format(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            SUPPORTED_EXTENSIONS.contains(&extension.to_lowercase().as_str())
        } else {
            false
        }
    }

    /// 画像ファイルリストをソートする
    /// 
    /// # Arguments
    /// * `images` - ソート対象の画像ファイルリスト
    fn sort_images(&self, images: &mut Vec<ImageFile>) {
        match self.config.sort_algorithm {
            SortAlgorithm::FileName => {
                images.sort_by(|a, b| a.name.cmp(&b.name));
                debug!("ファイル名でソートしました");
            }
            SortAlgorithm::CreatedTime => {
                images.sort_by(|a, b| a.created.cmp(&b.created));
                debug!("作成日時でソートしました");
            }
            SortAlgorithm::ModifiedTime => {
                images.sort_by(|a, b| a.modified.cmp(&b.modified));
                debug!("更新日時でソートしました");
            }
        }
    }

    /// 現在の画像ファイルを取得する
    /// 
    /// # Returns
    /// * `Option<&ImageFile>` - 現在の画像ファイル
    pub fn current_image(&self) -> Option<&ImageFile> {
        self.images.get(self.current_index)
    }

    /// 次の画像に移動する
    pub fn next_image(&mut self) {
        if !self.images.is_empty() {
            self.current_index = (self.current_index + 1) % self.images.len();
            debug!("次の画像に移動: インデックス {}", self.current_index);
        }
    }

    /// 前の画像に移動する
    pub fn previous_image(&mut self) {
        if !self.images.is_empty() {
            self.current_index = if self.current_index == 0 {
                self.images.len() - 1
            } else {
                self.current_index - 1
            };
            debug!("前の画像に移動: インデックス {}", self.current_index);
        }
    }

    /// 画像ファイルを読み込む
    /// 
    /// # Arguments
    /// * `path` - 画像ファイルのパス
    /// 
    /// # Returns
    /// * `Result<DynamicImage>` - 読み込まれた画像
    pub fn load_image(&self, path: &Path) -> Result<DynamicImage> {
        debug!("画像ファイルを読み込み中: {:?}", path);
        let img = image::open(path)?;
        debug!("画像ファイルの読み込み完了: {}x{}", img.width(), img.height());
        Ok(img)
    }

    /// 画像が空かどうかを確認する
    /// 
    /// # Returns
    /// * `bool` - 画像リストが空の場合は true
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// 画像の総数を取得する
    /// 
    /// # Returns
    /// * `usize` - 画像の総数
    pub fn len(&self) -> usize {
        self.images.len()
    }
}