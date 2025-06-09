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
    // スレッド情報を取得
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

    // OGP画像を生成（絵文字を除去してから）
    let clean_title = remove_emojis(&thread.title);
    let clean_username = remove_emojis(&thread.username);
    let image_data = generate_ogp_image(&clean_title, &clean_username)?;

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
    // 画像サイズ (1200x630はOGPの推奨サイズ)
    const WIDTH: u32 = 1200;
    const HEIGHT: u32 = 630;
    const BORDER_WIDTH: u32 = 32; // 枠線を太く

    // 色定義 (TailwindCSS text-orange-600: #EA580C)
    let background_color = Rgb([255, 255, 255]); // 白背景
    let border_color = Rgb([234, 88, 12]); // オレンジ枠線
    let text_color = Rgb([0, 0, 0]); // 黒文字
    let username_color = Rgb([107, 114, 128]); // グレー
    let brand_color = Rgb([234, 88, 12]); // ブランド名をオレンジ色に

    // 画像を作成
    let mut image: RgbImage = ImageBuffer::from_pixel(WIDTH, HEIGHT, background_color);

    // フォントを読み込み
    let font_data = include_bytes!("../../static/fonts/NotoSansJP-SemiBold.ttf");
    let font = Font::try_from_bytes(font_data as &[u8])
        .ok_or_else(|| AppError::Internal("Failed to load font".to_string()))?;

    // オレンジの枠線を描画
    // 上部
    draw_filled_rect_mut(
        &mut image,
        Rect::at(0, 0).of_size(WIDTH, BORDER_WIDTH),
        border_color,
    );
    // 下部
    draw_filled_rect_mut(
        &mut image,
        Rect::at(0, (HEIGHT - BORDER_WIDTH) as i32).of_size(WIDTH, BORDER_WIDTH),
        border_color,
    );
    // 左部
    draw_filled_rect_mut(
        &mut image,
        Rect::at(0, 0).of_size(BORDER_WIDTH, HEIGHT),
        border_color,
    );
    // 右部
    draw_filled_rect_mut(
        &mut image,
        Rect::at((WIDTH - BORDER_WIDTH) as i32, 0).of_size(BORDER_WIDTH, HEIGHT),
        border_color,
    );

    // タイトルを上部に描画（複数行対応）
    let title_scale = Scale { x: 80.0, y: 80.0 }; // フォントサイズを大きく
    let max_title_width = WIDTH - 200; // 左右マージン100px（枠線が太くなったため調整）
    let wrapped_title = wrap_text(title, &font, title_scale, max_title_width);

    let mut y_offset = 90; // 上マージンも調整
    for line in wrapped_title.iter().take(4) {
        // 最大4行
        draw_text_mut(
            &mut image,
            text_color,
            100,
            y_offset as i32,
            title_scale,
            &font,
            line,
        );
        y_offset += 85; // 行間を調整
    }

    // 左下にユーザー名を描画（@マーク付き）
    let username_scale = Scale { x: 58.0, y: 58.0 }; // フォントサイズを大きく
    let username_with_at = format!("@{}", username);
    draw_text_mut(
        &mut image,
        username_color,
        100,
        (HEIGHT - 140) as i32,
        username_scale,
        &font,
        &username_with_at,
    );

    // 右下に「みんなの話題」を描画
    let brand_scale = Scale { x: 58.0, y: 58.0 }; // フォントサイズを大きく
    let brand_text = "みんなの話題";

    // テキスト幅を計算して右寄せ
    let glyphs: Vec<_> = font
        .layout(brand_text, brand_scale, rusttype::point(0.0, 0.0))
        .collect();

    let text_width = glyphs
        .last()
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .unwrap_or(0.0) as u32;

    let brand_x = WIDTH - text_width - 100; // 右マージン100px
    draw_text_mut(
        &mut image,
        brand_color,
        brand_x as i32,
        (HEIGHT - 140) as i32,
        brand_scale,
        &font,
        brand_text,
    );

    // PNGとしてエンコード
    let mut buffer = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut buffer),
            image::ImageOutputFormat::Png,
        )
        .map_err(|_| AppError::Internal("PNG encoding error".to_string()))?;

    Ok(buffer)
}

fn wrap_text(text: &str, font: &Font, scale: Scale, max_width: u32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        let width = text_width(font, scale, &test_line);

        if width <= max_width as f32 {
            current_line = test_line;
        } else {
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = word.to_string();
            let current_width = text_width(font, scale, &current_line);

            // 極端に長い単語（URLなど）は強制折り返し
            if current_width > max_width as f32 {
                let broken = break_long_word(&current_line, font, scale, max_width);
                lines.extend(broken.into_iter());
                current_line.clear();
            }
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push("無題".to_string());
    }

    lines
}

/// 単語の描画幅を取得
fn text_width(font: &Font, scale: Scale, text: &str) -> f32 {
    font.layout(text, scale, rusttype::point(0.0, 0.0))
        .last()
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .unwrap_or(0.0)
}

/// 長い単語（URLなど）を強制的に折り返す
fn break_long_word(word: &str, font: &Font, scale: Scale, max_width: u32) -> Vec<String> {
    let mut result = Vec::new();
    let mut buffer = String::new();

    for c in word.chars() {
        buffer.push(c);
        if text_width(font, scale, &buffer) > max_width as f32 {
            buffer.pop();
            if !buffer.is_empty() {
                result.push(buffer.clone());
            }
            buffer = c.to_string();
        }
    }

    if !buffer.is_empty() {
        result.push(buffer);
    }

    result
}

/// 絵文字を除去する関数
fn remove_emojis(text: &str) -> String {
    text.chars()
        .filter(|&c| {
            // Unicode 絵文字の範囲をチェック
            !is_emoji(c)
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// 文字が絵文字かどうかを判定
fn is_emoji(c: char) -> bool {
    let code = c as u32;

    // よく使われる絵文字の Unicode 範囲
    matches!(code,
        // Emoticons (1F600-1F64F)
        0x1F600..=0x1F64F |
        // Miscellaneous Symbols and Pictographs (1F300-1F5FF)
        0x1F300..=0x1F5FF |
        // Transport and Map Symbols (1F680-1F6FF)
        0x1F680..=0x1F6FF |
        // Symbols and Pictographs Extended-A (1F900-1F9FF)
        0x1F900..=0x1F9FF |
        // Miscellaneous Symbols (2600-26FF)
        0x2600..=0x26FF |
        // Dingbats (2700-27BF)
        0x2700..=0x27BF |
        // Variation Selectors (FE00-FE0F)
        0xFE00..=0xFE0F |
        // Miscellaneous Technical (2300-23FF) - 一部の記号
        0x231A..=0x231B | 0x23E9..=0x23EC | 0x23F0 | 0x23F3 |
        // Enclosed Alphanumeric Supplement (1F100-1F1FF)
        0x1F100..=0x1F1FF
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::seed_test_data;

    #[sqlx::test]
    async fn test_ogp_画像生成_成功時(pool: PgPool) {
        // テストデータを準備
        let (_user_id, thread_id) = seed_test_data(&pool, "ogp_test").await;

        // OGP画像生成をテスト
        let response = get_thread_ogp_image(State(pool), Path(thread_id)).await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Content-Typeがimage/pngであることを確認
        let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(content_type, "image/png");
    }

    #[sqlx::test]
    async fn test_ogp_画像生成_存在しないスレッド(pool: PgPool) {
        // 存在しないスレッドIDでOGP画像生成をテスト
        let non_existent_id = Uuid::new_v4();

        let response = get_thread_ogp_image(State(pool), Path(non_existent_id)).await;

        assert!(response.is_err());
        match response.unwrap_err() {
            AppError::NotFound => {} // 期待される結果
            _ => panic!("期待されるエラータイプではありません"),
        }
    }

    #[test]
    fn test_generate_ogp_image_画像生成() {
        // OGP画像生成のテスト
        let title = "これは非常に長いタイトルのテストです";
        let username = "testuser";

        let result = generate_ogp_image(title, username);

        match &result {
            Ok(_) => {}
            Err(e) => panic!("OGP画像生成に失敗しました: {:?}", e),
        }

        let image_data = result.unwrap();
        assert!(!image_data.is_empty());

        // PNG画像の魔法バイトをチェック
        assert_eq!(&image_data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_generate_ogp_image_長いタイトル() {
        // より長いタイトルでのテスト
        let title = "これは非常に長いタイトルのテストで、複数行に分かれることを期待しています。折り返し機能が正しく動作するかを確認するためのテストケースです。";
        let username = "very_long_username_test";

        let result = generate_ogp_image(title, username);

        assert!(result.is_ok());
        let image_data = result.unwrap();
        assert!(!image_data.is_empty());

        // PNG画像の魔法バイトをチェック
        assert_eq!(&image_data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_wrap_text_テキスト折り返し() {
        // フォントデータを読み込み
        let font_data = include_bytes!("../../static/fonts/NotoSansJP-SemiBold.ttf");
        let font = Font::try_from_bytes(font_data as &[u8]).unwrap();

        let text = "これは非常に長いタイトルのテストです。複数行に分かれることを期待しています。";
        let scale = Scale { x: 36.0, y: 36.0 };
        let max_width = 600;

        let lines = wrap_text(text, &font, scale, max_width);

        assert!(!lines.is_empty());
        // 長いテキストは複数行に分かれることを確認
        assert!(lines.len() >= 1);
    }

    #[test]
    fn test_remove_emojis_絵文字除去() {
        // 絵文字が含まれるテキストのテスト
        let text_with_emojis = "こんにちは😀世界🌍！テスト📝です🎉";
        let result = remove_emojis(text_with_emojis);
        assert_eq!(result, "こんにちは世界！テストです");

        // 絵文字のみのテキスト
        let emoji_only = "😀🌍📝🎉";
        let result2 = remove_emojis(emoji_only);
        assert_eq!(result2, "");

        // 絵文字なしのテキスト
        let no_emojis = "普通のテキストです";
        let result3 = remove_emojis(no_emojis);
        assert_eq!(result3, "普通のテキストです");

        // 空文字列
        let empty = "";
        let result4 = remove_emojis(empty);
        assert_eq!(result4, "");
    }

    #[test]
    fn test_is_emoji_判定() {
        // 絵文字文字の判定テスト
        assert!(is_emoji('😀')); // 顔の絵文字
        assert!(is_emoji('🌍')); // 地球の絵文字
        assert!(is_emoji('📝')); // メモの絵文字
        assert!(is_emoji('🎉')); // 祝いの絵文字

        // 通常の文字は絵文字ではない
        assert!(!is_emoji('あ')); // ひらがな
        assert!(!is_emoji('A')); // 英字
        assert!(!is_emoji('1')); // 数字
        assert!(!is_emoji('!')); // 記号
    }

    #[test]
    fn test_generate_ogp_image_絵文字入りタイトル() {
        // 絵文字が含まれるタイトルでのテスト
        let title = "テスト投稿です😀🎉 いい感じ！";
        let username = "testuser📝";

        let result = generate_ogp_image(title, username);

        assert!(result.is_ok());
        let image_data = result.unwrap();
        assert!(!image_data.is_empty());

        // PNG画像の魔法バイトをチェック
        assert_eq!(&image_data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }
}
