# rs_fast_image_viewer

WebP対応の高速画像ビューアーアプリケーション

## 概要

Rust 製の高速画像ビューアーです。JPEG、PNG、WEBP に対応し、ディレクトリ指定時にはソートされた画像リストを順次閲覧できます。

## 対応画像フォーマット

- JPEG (.jpg, .jpeg)
- PNG (.png)
- WebP (.webp)

## 機能

- 高速起動と画像表示
- ディレクトリ内の画像ファイル一覧表示
- 設定ファイルによるカスタマイズ
- 画像サイズに応じた自動ウィンドウサイズ調整
- キーボードショートカット対応

## システム要件

- Windows 11
- Rust 1.82+ (MSRVに準拠)
- DirectX 12, Vulkan, またはOpenGL ES対応のGPU

## インストール

### Gitから直接インストール

```bash
cargo install --git https://github.com/kznagamori/rs_fast_image_viewer
```

### ローカルビルド

```bash
git clone https://github.com/kznagamori/rs_fast_image_viewer
cd rs_fast_image_viewer
cargo build --release
```

## 使用方法

### 基本的な使用方法

```bash
# 画像ファイルを指定して開く
rs_fast_image_viewer path/to/image.webp

# ディレクトリを指定して最初の画像を開く
rs_fast_image_viewer path/to/image/directory
```

### キーボードショートカット

- `→` または `X`: 次の画像へ
- `←` または `Z`: 前の画像へ
- `Enter` または `Escape`: アプリケーション終了
- `Alt+F4`: アプリケーション終了

## 設定ファイル

実行ファイルと同じディレクトリに `rs_fast_image_viewer.toml` ファイルが自動作成されます。

```toml
# 最小ウィンドウサイズ (幅, 高さ)
min_window_size = [800, 600]

# ソートアルゴリズム ("FileName", "CreatedTime", "ModifiedTime")
sort_algorithm = "FileName"
```

### ソートアルゴリズム

- `FileName`: ファイル名でソート
- `CreatedTime`: 作成日時でソート  
- `ModifiedTime`: 更新日時でソート

## 画像表示の動作

1. 画像が最小ウィンドウサイズより小さい場合、アスペクト比を保持して拡大表示
2. 画像が最小ウィンドウサイズより大きい場合、その大きさで表示
3. 画像がデスクトップ解像度を超える場合、アスペクト比を保持して縮小表示

## 技術仕様

### 開発環境

- **言語**: Rust 2024 Edition
- **ツールチェイン**: x86_64-pc-windows-gnu
- **OS**: Windows 11

### 主要依存関係

- `wgpu` 26.0+ - GPU描画エンジン
- `winit` 0.30+ - ウィンドウ管理
- `image` 0.25+ - 画像処理（WebP対応）
- `clap` 4.0+ - コマンドライン引数処理
- `pollster` - 非同期処理のブロック実行
- `bytemuck` - バイナリデータ変換
- `log`, `fern` - ログ処理
- `serde`, `toml` - 設定ファイル処理

### Windows固有の依存関係

- `winapi` 0.3+ - Windows API（デスクトップ解像度取得用）

## アーキテクチャ

### プロジェクト構成

```
rs_fast_image_viewer/
├── Cargo.toml              # プロジェクト設定
├── README.md               # このファイル
├── shaders/
│   └── image.wgsl          # WGSL シェーダー
└── src/
    ├── main.rs             # メインエントリーポイント
    ├── config.rs           # 設定ファイル処理
    ├── image_handler.rs    # 画像ファイル管理
    └── viewer.rs           # GUI・描画処理
```

### 主要コンポーネント

- **Config**: 設定ファイルの読み込み・保存
- **ImageHandler**: 画像ファイルの検索・管理・読み込み
- **ImageViewer**: wgpu/winitベースのGUI・描画処理

### レンダリングパイプライン

1. WGPU インスタンス作成
2. GPU アダプター選択
3. デバイス・キュー初期化
4. サーフェス設定
5. シェーダー・パイプライン作成
6. 画像テクスチャ生成
7. レンダリング実行

## 開発者向け情報

### ビルド要件

- Rust 1.82以上
- wgpu対応GPU（DirectX 12, Vulkan, OpenGL ES）
- Windows SDK (winapi使用のため)

### デバッグビルド

```bash
cargo build
```

### リリースビルド

```bash
cargo build --release
```

### ログレベル設定

環境変数でログレベルを制御可能：

```bash
set RUST_LOG=debug
rs_fast_image_viewer image.webp
```

## トラブルシューティング

### よくある問題

**Q: WebP画像が表示されない**
A: `image` crateのWebP機能が有効になっていることを確認してください。

**Q: GPUエラーが発生する**
A: 最新のGPUドライバーがインストールされていることを確認してください。

**Q: ウィンドウが表示されない**
A: デスクトップ解像度の取得に失敗している可能性があります。設定ファイルでウィンドウサイズを調整してください。

### エラーログ

アプリケーションは標準出力にログを出力します。問題が発生した場合は、ログを確認してください。

## ライセンス

MIT License

## 貢献

プルリクエストやIssueの報告を歓迎します。

## 作者

© 2025 kznagamori

## 更新履歴

### v0.1.0
- 初回リリース
- WebP, JPEG, PNG対応
- 基本的な画像ビューア機能
- 設定ファイル対応
- Windows対応
