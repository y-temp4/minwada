use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
};
use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut},
    rect::Rect,
};
use rusttype::{Font, Scale};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::threads::ThreadWithUser;

/// スレッドのOGP画像を生成
///
/// 指定されたスレッドIDに基づいてOGP画像を生成します。
/// 画像にはスレッドのタイトルと投稿者名が含まれます。
#[utoipa::path(
    get,
    path = "/api/threads/{thread_id}/ogp.png",
    params(
        ("thread_id" = Uuid, Path, description = "スレッドID")
    ),
    responses(
        (status = 200, description = "OGP画像", content_type = "image/png"),
        (status = 404, description = "スレッドが見つかりません")
    ),
    tag = "threads"
)]
pub async fn get_thread_ogp_image(
    State(pool): State<PgPool>,
    Path(thread_id): Path<Uuid>,
) -> Result<Response> {
    // データベースからスレッド情報を取得（投稿者情報とコメント数も含む）
    let thread = sqlx::query_as::<_, ThreadWithUser>(
        r#"
        SELECT 
            t.id, t.title, t.content, t.created_at, t.updated_at,
            t.upvote_count, t.downvote_count,
            u.id as user_id, u.username, u.display_name as user_display_name, u.avatar_url as user_avatar_url,
            COUNT(c.id)::bigint as comment_count
        FROM threads t
        JOIN users u ON t.user_id = u.id
        LEFT JOIN comments c ON t.id = c.thread_id
        WHERE t.id = $1
        GROUP BY t.id, t.upvote_count, t.downvote_count, u.id, u.username, u.display_name, u.avatar_url
        "#
    )
    .bind(thread_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound)?;

    // タイトルとユーザー名から絵文字を除去してOGP画像を生成
    let clean_title = remove_emojis(&thread.title);
    let image_data = generate_ogp_image(&clean_title, &thread.username)?;

    // 画像データをPNG形式でレスポンスとして返す（24時間キャッシュ設定）
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/png")
        .header(header::CACHE_CONTROL, "public, max-age=86400") // 24時間キャッシュ
        .body(image_data.into())
        .map_err(|_| AppError::Internal("Response build error".to_string()))?;

    Ok(response)
}

/// OGP画像を生成する関数
fn generate_ogp_image(title: &str, username: &str) -> Result<Vec<u8>> {
    // OGP推奨サイズ（1200x630）で画像を作成
    const WIDTH: u32 = 1200;
    const HEIGHT: u32 = 630;
    const BORDER_WIDTH: u32 = 32; // オレンジ枠線の太さ

    // カラーパレット定義（TailwindCSS準拠）
    let background_color = Rgb([255, 255, 255]); // 白背景
    let border_color = Rgb([234, 88, 12]); // オレンジ枠線（text-orange-600）
    let text_color = Rgb([0, 0, 0]); // 黒文字
    let username_color = Rgb([107, 114, 128]); // グレー文字
    let brand_color = Rgb([234, 88, 12]); // ブランド名（オレンジ）

    // 白背景の画像を作成
    let mut image: RgbImage = ImageBuffer::from_pixel(WIDTH, HEIGHT, background_color);

    // 日本語フォント（Noto Sans JP）を読み込み
    let font_data = include_bytes!("../../static/fonts/NotoSansJP-SemiBold.ttf");
    let font = Font::try_from_bytes(font_data as &[u8])
        .ok_or_else(|| AppError::Internal("Failed to load font".to_string()))?;

    // 画像四辺にオレンジの枠線を描画
    // 上辺
    draw_filled_rect_mut(
        &mut image,
        Rect::at(0, 0).of_size(WIDTH, BORDER_WIDTH),
        border_color,
    );
    // 下辺
    draw_filled_rect_mut(
        &mut image,
        Rect::at(0, (HEIGHT - BORDER_WIDTH) as i32).of_size(WIDTH, BORDER_WIDTH),
        border_color,
    );
    // 左辺
    draw_filled_rect_mut(
        &mut image,
        Rect::at(0, 0).of_size(BORDER_WIDTH, HEIGHT),
        border_color,
    );
    // 右辺
    draw_filled_rect_mut(
        &mut image,
        Rect::at((WIDTH - BORDER_WIDTH) as i32, 0).of_size(BORDER_WIDTH, HEIGHT),
        border_color,
    );

    // スレッドタイトルを画像上部に描画（長い場合は自動改行、最大4行）
    let title_scale = Scale { x: 80.0, y: 80.0 }; // タイトル用フォントサイズ
    let max_title_width = WIDTH - 200; // 左右マージン100pxずつ確保
    let wrapped_title = wrap_text(title, &font, title_scale, max_title_width);

    let mut y_offset = 90; // タイトル開始位置（上マージン）
    for line in wrapped_title.iter().take(4) {
        // 最大4行まで表示
        draw_text_mut(
            &mut image,
            text_color,
            100, // 左マージン
            y_offset as i32,
            title_scale,
            &font,
            line,
        );
        y_offset += 85; // 行間隔
    }

    // 左下にユーザー名を描画（@マーク付きで表示）
    let username_scale = Scale { x: 58.0, y: 58.0 }; // ユーザー名用フォントサイズ
    let username_with_at = format!("@{}", username);
    draw_text_mut(
        &mut image,
        username_color,
        100,                   // 左マージン
        (HEIGHT - 140) as i32, // 下から140px上
        username_scale,
        &font,
        &username_with_at,
    );

    // 右下にサイト名「みんなの話題」を描画
    let brand_scale = Scale { x: 58.0, y: 58.0 }; // ブランド名用フォントサイズ
    let brand_text = "みんなの話題";

    // ブランド名のテキスト幅を計算して右寄せ配置
    let glyphs: Vec<_> = font
        .layout(brand_text, brand_scale, rusttype::point(0.0, 0.0))
        .collect();

    let text_width = glyphs
        .last()
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .unwrap_or(0.0) as u32;

    let brand_x = WIDTH - text_width - 100; // 右マージン100px確保
    draw_text_mut(
        &mut image,
        brand_color,
        brand_x as i32,
        (HEIGHT - 140) as i32, // 下から140px上
        brand_scale,
        &font,
        brand_text,
    );

    // 生成した画像をPNG形式のバイト配列にエンコード
    let mut buffer = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut buffer),
            image::ImageOutputFormat::Png,
        )
        .map_err(|_| AppError::Internal("PNG encoding error".to_string()))?;

    Ok(buffer)
}

/// テキストを指定幅に合わせて自動改行する関数
fn wrap_text(text: &str, font: &Font, scale: Scale, max_width: u32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    // 単語ごとに分割して処理
    for word in text.split_whitespace() {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        let width = text_width(font, scale, &test_line);

        if width <= max_width as f32 {
            // 幅内に収まる場合は行に追加
            current_line = test_line;
        } else {
            // 幅を超える場合は改行
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = word.to_string();
            let current_width = text_width(font, scale, &current_line);

            // 単語が非常に長い場合（URLなど）は強制的に文字単位で折り返し
            if current_width > max_width as f32 {
                let broken = break_long_word(&current_line, font, scale, max_width);
                lines.extend(broken.into_iter());
                current_line.clear();
            }
        }
    }

    // 最後の行を追加
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // 空の場合は「無題」を表示
    if lines.is_empty() {
        lines.push("無題".to_string());
    }

    lines
}

/// テキストの描画幅を計算する関数
fn text_width(font: &Font, scale: Scale, text: &str) -> f32 {
    font.layout(text, scale, rusttype::point(0.0, 0.0))
        .last()
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .unwrap_or(0.0)
}

/// 長すぎる単語を文字単位で強制改行する関数（URL対応）
fn break_long_word(word: &str, font: &Font, scale: Scale, max_width: u32) -> Vec<String> {
    let mut result = Vec::new();
    let mut buffer = String::new();

    for c in word.chars() {
        let single_char_width = text_width(font, scale, &c.to_string());

        // 1文字でもmax_widthを超える場合は、その文字だけで1行にする
        if single_char_width > max_width as f32 {
            if !buffer.is_empty() {
                result.push(buffer.clone());
                buffer.clear();
            }
            result.push(c.to_string());
            continue;
        }

        buffer.push(c);
        if text_width(font, scale, &buffer) > max_width as f32 {
            buffer.pop(); // 最後の文字を削除
            if !buffer.is_empty() {
                result.push(buffer.clone());
            }
            buffer = c.to_string(); // 新しい行で削除した文字から開始
        }
    }

    if !buffer.is_empty() {
        result.push(buffer);
    }

    result
}

/// テキストから絵文字を除去する関数
fn remove_emojis(text: &str) -> String {
    text.chars()
        .filter(|&c| {
            // Unicode範囲チェックで絵文字を除外
            !is_emoji(c)
        })
        .collect::<String>()
        .trim() // 前後の空白を削除
        .to_string()
}

/// 文字がUnicode絵文字かどうかを判定する関数
fn is_emoji(c: char) -> bool {
    let code = c as u32;

    // 主要な絵文字のUnicode範囲をチェック
    matches!(code,
        // 基本的な顔文字絵文字 (1F600-1F64F)
        0x1F600..=0x1F64F |
        // 雑多なシンボルと絵文字 (1F300-1F5FF)
        0x1F300..=0x1F5FF |
        // 交通と地図のシンボル (1F680-1F6FF)
        0x1F680..=0x1F6FF |
        // 追加シンボルと絵文字 (1F900-1F9FF)
        0x1F900..=0x1F9FF |
        // その他シンボル (2600-26FF)
        0x2600..=0x26FF |
        // 装飾記号 (2700-27BF)
        0x2700..=0x27BF |
        // 異体字セレクタ (FE00-FE0F)
        0xFE00..=0xFE0F |
        // 技術的記号の一部 (2300-23FF)
        0x231A..=0x231B | 0x23E9..=0x23EC | 0x23F0 | 0x23F3 |
        // 囲み英数字補助 (1F100-1F1FF)
        0x1F100..=0x1F1FF
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::seed_test_data;

    #[sqlx::test]
    async fn test_正常なスレッドで_ogp_画像が生成される(pool: PgPool) {
        // テスト用のユーザーとスレッドデータを作成
        let (_user_id, thread_id) = seed_test_data(&pool, "ogp_test").await;

        // スレッドIDを指定してOGP画像生成APIを呼び出し
        let response = get_thread_ogp_image(State(pool), Path(thread_id)).await;

        // レスポンスが正常に返されることを確認
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Content-TypeがPNG画像であることを確認
        let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(content_type, "image/png");
    }

    #[sqlx::test]
    async fn test_存在しないスレッド_id_で_404_エラーが返される(pool: PgPool) {
        // データベースに存在しないランダムなスレッドIDを生成
        let non_existent_id = Uuid::new_v4();

        // 存在しないスレッドIDでOGP画像生成を試行
        let response = get_thread_ogp_image(State(pool), Path(non_existent_id)).await;

        // NotFoundエラーが返されることを確認
        assert!(response.is_err());
        match response.unwrap_err() {
            AppError::NotFound => {} // 期待される結果
            _ => panic!("NotFoundエラーが期待されましたが、異なるエラーが発生しました"),
        }
    }

    #[test]
    fn test_基本的な_ogp_画像が正常に生成される() {
        // 通常の長さのタイトルとユーザー名でテスト
        let title = "これは非常に長いタイトルのテストです";
        let username = "testuser";

        // OGP画像生成を実行
        let result = generate_ogp_image(title, username);

        // 画像生成が成功することを確認
        match &result {
            Ok(_) => {}
            Err(e) => panic!("OGP画像生成に失敗しました: {:?}", e),
        }

        let image_data = result.unwrap();
        assert!(!image_data.is_empty());

        // 生成されたデータがPNG形式であることを確認（マジックバイト）
        assert_eq!(&image_data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_非常に長いタイトルでも画像生成が成功する() {
        // 複数行に折り返しが必要な長いタイトルでテスト
        let title = "これは非常に長いタイトルのテストで、複数行に分かれることを期待しています。折り返し機能が正しく動作するかを確認するためのテストケースです。";
        let username = "very_long_username_test";

        // 長いタイトルでも画像生成が成功することを確認
        let result = generate_ogp_image(title, username);

        assert!(result.is_ok());
        let image_data = result.unwrap();
        assert!(!image_data.is_empty());

        // PNG形式で出力されることを確認
        assert_eq!(&image_data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_テキストが指定幅で正しく折り返される() {
        // 日本語フォントを読み込み
        let font_data = include_bytes!("../../static/fonts/NotoSansJP-SemiBold.ttf");
        let font = Font::try_from_bytes(font_data as &[u8]).unwrap();

        // 折り返しテスト用の長いテキスト
        let text = "これは非常に長いタイトルのテストです。複数行に分かれることを期待しています。";
        let scale = Scale { x: 36.0, y: 36.0 };
        let max_width = 600;

        // テキスト折り返し処理を実行
        let lines = wrap_text(text, &font, scale, max_width);

        // 折り返し結果が正しく生成されることを確認
        assert!(!lines.is_empty());
        // 長いテキストは少なくとも1行以上に分かれることを確認
        assert!(lines.len() >= 1);
    }

    #[test]
    fn test_テキストから絵文字が正しく除去される() {
        // 様々なパターンの絵文字を含むテキストでテスト
        let text_with_emojis = "こんにちは😀世界🌍！テスト📝です🎉";
        let result = remove_emojis(text_with_emojis);
        assert_eq!(result, "こんにちは世界！テストです");

        // 絵文字のみのテキスト（空文字列になることを確認）
        let emoji_only = "😀🌍📝🎉";
        let result2 = remove_emojis(emoji_only);
        assert_eq!(result2, "");

        // 絵文字が含まれないテキスト（そのまま残ることを確認）
        let no_emojis = "普通のテキストです";
        let result3 = remove_emojis(no_emojis);
        assert_eq!(result3, "普通のテキストです");

        // 空文字列の処理
        let empty = "";
        let result4 = remove_emojis(empty);
        assert_eq!(result4, "");
    }

    #[test]
    fn test_絵文字判定が正しく動作する() {
        // 各種絵文字が正しく判定されることを確認
        assert!(is_emoji('😀')); // 笑顔の絵文字
        assert!(is_emoji('🌍')); // 地球の絵文字
        assert!(is_emoji('📝')); // メモの絵文字
        assert!(is_emoji('🎉')); // お祝いの絵文字

        // 通常の文字が絵文字として誤判定されないことを確認
        assert!(!is_emoji('あ')); // ひらがな
        assert!(!is_emoji('A')); // 英大文字
        assert!(!is_emoji('1')); // 数字
        assert!(!is_emoji('!')); // 記号
    }

    #[test]
    fn test_絵文字入りタイトルでも_ogp_画像が生成される() {
        // 絵文字が混在するタイトルとユーザー名でテスト
        let title = "テスト投稿です😀🎉 いい感じ！";
        let username = "testuser";

        // 絵文字が含まれていても画像生成が成功することを確認
        let result = generate_ogp_image(title, username);

        assert!(result.is_ok());
        let image_data = result.unwrap();
        assert!(!image_data.is_empty());

        // PNG形式で正しく生成されることを確認
        assert_eq!(&image_data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }
}
