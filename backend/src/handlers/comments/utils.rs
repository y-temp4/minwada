use crate::models::comments::{CommentResponse, CommentWithUser};

pub fn build_comment_tree(comments: Vec<CommentWithUser>) -> Vec<CommentResponse> {
    // 1. 最初にすべてのコメントをCommentResponseに変換
    let mut all_comments: Vec<CommentResponse> =
        comments.into_iter().map(|c| c.to_response()).collect();

    // 2. created_at順でソート（決定的な順序を保証）
    all_comments.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    // 3. ルートコメント（parent_id = None）と子コメントを分離
    let mut root_comments: Vec<CommentResponse> = Vec::new();
    let mut child_comments: Vec<CommentResponse> = Vec::new();

    for comment in all_comments {
        if comment.parent_id.is_none() {
            root_comments.push(comment);
        } else {
            child_comments.push(comment);
        }
    }

    // 4. 子コメントをルートコメントの適切な位置に配置
    fn add_children_to_parent(parent: &mut CommentResponse, children: &[CommentResponse]) {
        let mut direct_children: Vec<CommentResponse> = children
            .iter()
            .filter(|child| child.parent_id == Some(parent.id))
            .cloned()
            .collect();

        // 子コメントもcreated_at順でソート
        direct_children.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        for mut child in direct_children {
            // 再帰的に孫コメントも追加
            add_children_to_parent(&mut child, children);
            parent.replies.push(child);
            parent.reply_count += 1;
        }
    }

    // 5. 各ルートコメントに子コメントを追加
    for root_comment in &mut root_comments {
        add_children_to_parent(root_comment, &child_comments);
    }

    root_comments
}