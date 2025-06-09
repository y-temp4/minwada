use crate::models::comments::{CommentResponse, CommentWithUser};
use std::collections::HashMap;
use uuid::Uuid;

pub fn build_comment_tree(comments: Vec<CommentWithUser>) -> Vec<CommentResponse> {
    if comments.is_empty() {
        return Vec::new();
    }

    // 1. すべてのコメントをCommentResponseに変換し、created_at順でソート
    let mut all_comments: Vec<CommentResponse> =
        comments.into_iter().map(|c| c.to_response()).collect();
    all_comments.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    // 2. 親IDごとに子コメントをグループ化するHashMapを構築
    let mut children_map: HashMap<Uuid, Vec<CommentResponse>> = HashMap::new();
    let mut root_comments: Vec<CommentResponse> = Vec::new();

    for comment in all_comments {
        if let Some(parent_id) = comment.parent_id {
            children_map
                .entry(parent_id)
                .or_insert_with(Vec::new)
                .push(comment);
        } else {
            root_comments.push(comment);
        }
    }

    // 3. 各コメントに子コメントを再帰的に追加
    fn build_replies(
        comment: &mut CommentResponse,
        children_map: &HashMap<Uuid, Vec<CommentResponse>>,
    ) {
        if let Some(children) = children_map.get(&comment.id) {
            for mut child in children.clone() {
                build_replies(&mut child, children_map);
                comment.reply_count += 1;
                comment.replies.push(child);
            }
        }
    }

    // 4. 各ルートコメントに子コメントを追加
    for root_comment in &mut root_comments {
        build_replies(root_comment, &children_map);
    }

    root_comments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::comments::CommentWithUser;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    fn create_test_comment(
        id: Uuid,
        parent_id: Option<Uuid>,
        content: &str,
        created_at: DateTime<Utc>,
    ) -> CommentWithUser {
        CommentWithUser {
            id,
            // thread_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            parent_id,
            content: content.to_string(),
            created_at,
            updated_at: created_at,
            username: "testuser".to_string(),
            user_display_name: Some("Test User".to_string()),
            user_avatar_url: None,
        }
    }

    #[test]
    fn test_空のコメントリストで空の結果を返す() {
        let comments = Vec::new();
        let result = build_comment_tree(comments);
        assert!(result.is_empty());
    }

    #[test]
    fn test_単一のルートコメントを正しく処理() {
        let base_time = Utc::now();
        let comment1_id = Uuid::new_v4();

        let comments = vec![create_test_comment(
            comment1_id,
            None,
            "Root comment",
            base_time,
        )];

        let result = build_comment_tree(comments);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, comment1_id);
        assert_eq!(result[0].content, "Root comment");
        assert_eq!(result[0].replies.len(), 0);
        assert_eq!(result[0].reply_count, 0);
    }

    #[test]
    fn test_階層コメントツリーを正しく構築() {
        let base_time = Utc::now();
        let root_id = Uuid::new_v4();
        let child1_id = Uuid::new_v4();
        let child2_id = Uuid::new_v4();
        let grandchild_id = Uuid::new_v4();

        let comments = vec![
            create_test_comment(root_id, None, "Root", base_time),
            create_test_comment(
                child1_id,
                Some(root_id),
                "Child 1",
                base_time + chrono::Duration::minutes(1),
            ),
            create_test_comment(
                child2_id,
                Some(root_id),
                "Child 2",
                base_time + chrono::Duration::minutes(2),
            ),
            create_test_comment(
                grandchild_id,
                Some(child1_id),
                "Grandchild",
                base_time + chrono::Duration::minutes(3),
            ),
        ];

        let result = build_comment_tree(comments);

        assert_eq!(result.len(), 1);
        let root = &result[0];
        assert_eq!(root.id, root_id);
        assert_eq!(root.replies.len(), 2);
        assert_eq!(root.reply_count, 2);

        // 子コメントの順序確認（created_at順）
        assert_eq!(root.replies[0].id, child1_id);
        assert_eq!(root.replies[1].id, child2_id);

        // 孫コメントの確認
        let child1 = &root.replies[0];
        assert_eq!(child1.replies.len(), 1);
        assert_eq!(child1.reply_count, 1);
        assert_eq!(child1.replies[0].id, grandchild_id);
    }

    #[test]
    fn test_複数のルートコメントと時系列順序() {
        let base_time = Utc::now();
        let root1_id = Uuid::new_v4();
        let root2_id = Uuid::new_v4();
        let child1_id = Uuid::new_v4();

        let comments = vec![
            create_test_comment(
                root2_id,
                None,
                "Root 2",
                base_time + chrono::Duration::minutes(2),
            ),
            create_test_comment(
                child1_id,
                Some(root1_id),
                "Child of Root 1",
                base_time + chrono::Duration::minutes(3),
            ),
            create_test_comment(root1_id, None, "Root 1", base_time),
        ];

        let result = build_comment_tree(comments);

        assert_eq!(result.len(), 2);
        // created_at順でソートされていることを確認
        assert_eq!(result[0].id, root1_id);
        assert_eq!(result[1].id, root2_id);

        // Root 1が子コメントを持つことを確認
        assert_eq!(result[0].replies.len(), 1);
        assert_eq!(result[0].replies[0].id, child1_id);

        // Root 2は子コメントなし
        assert_eq!(result[1].replies.len(), 0);
    }

    #[test]
    fn test_深い階層のコメントツリー() {
        let base_time = Utc::now();
        let ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

        let comments = vec![
            create_test_comment(ids[0], None, "Level 0", base_time),
            create_test_comment(
                ids[1],
                Some(ids[0]),
                "Level 1",
                base_time + chrono::Duration::minutes(1),
            ),
            create_test_comment(
                ids[2],
                Some(ids[1]),
                "Level 2",
                base_time + chrono::Duration::minutes(2),
            ),
            create_test_comment(
                ids[3],
                Some(ids[2]),
                "Level 3",
                base_time + chrono::Duration::minutes(3),
            ),
            create_test_comment(
                ids[4],
                Some(ids[3]),
                "Level 4",
                base_time + chrono::Duration::minutes(4),
            ),
        ];

        let result = build_comment_tree(comments);

        assert_eq!(result.len(), 1);

        // 各レベルの深さを確認
        let mut current = &result[0];
        for i in 0..4 {
            assert_eq!(current.id, ids[i]);
            assert_eq!(current.replies.len(), 1);
            assert_eq!(current.reply_count, 1);
            current = &current.replies[0];
        }

        // 最深レベル
        assert_eq!(current.id, ids[4]);
        assert_eq!(current.replies.len(), 0);
        assert_eq!(current.reply_count, 0);
    }
}
