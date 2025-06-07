"use client";

import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Skeleton } from "@/components/ui/skeleton";
import {
  RefreshCw,
  MessageSquare,
  User,
  Clock,
  Reply,
  Trash2,
} from "lucide-react";
import { ReplyCommentForm } from "@/components/reply-comment-form";
import { useAuth } from "@/providers/auth-provider";
import { CommentResponse, CommentListResponse } from "@/generated/schemas";
import { useDeleteComment, getGetCommentsQueryKey } from "@/generated/api";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import Link from "next/link";

interface CommentListProps {
  threadId: string;
  commentsResponse?: CommentListResponse;
  isLoading: boolean;
  error?: unknown;
}

interface CommentItemProps {
  comment: CommentResponse;
  threadId: string;
  depth?: number;
  maxDepth?: number;
}

function CommentItem({
  comment,
  threadId,
  depth = 0,
  maxDepth = 3,
}: CommentItemProps) {
  const { isAuthenticated, user } = useAuth();
  const queryClient = useQueryClient();
  const [showReplyForm, setShowReplyForm] = useState(false);
  const [showReplies, setShowReplies] = useState(false);
  const [newReplyId, setNewReplyId] = useState<string | null>(null);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);

  const { mutate: deleteComment } = useDeleteComment();

  const handleDeleteComment = () => {
    deleteComment(
      { id: comment.id },
      {
        onSuccess: () => {
          console.log("Comment deletion successful");
          // コメント一覧のキャッシュを無効化
          queryClient.invalidateQueries({
            queryKey: [...getGetCommentsQueryKey(threadId)],
          });
          toast.success("コメントが削除されました");
          setShowDeleteDialog(false);
        },
        onError: (error) => {
          console.error("Comment deletion failed:", error);
          toast.error("コメントの削除に失敗しました");
        },
      }
    );
  };

  // 新しい返信が追加された後のスクロール処理
  useEffect(() => {
    if (newReplyId && showReplies) {
      const timer = setTimeout(() => {
        const replyElement = document.getElementById(`comment-${newReplyId}`);
        if (replyElement) {
          replyElement.scrollIntoView({
            behavior: "smooth",
            block: "center",
          });
          // 一度スクロールしたらIDをクリア
          setNewReplyId(null);
        }
      }, 100); // DOM更新を待つため少し遅延

      return () => clearTimeout(timer);
    }
  }, [newReplyId, showReplies, comment.replies]);

  const handleReplyClick = () => {
    setShowReplyForm(!showReplyForm);
  };

  const handleCancelReply = () => {
    setShowReplyForm(false);
  };

  const handleReplySuccess = (replyId?: string) => {
    setShowReplyForm(false);
    setShowReplies(true); // 返信ツリーを開く
    if (replyId) {
      setNewReplyId(replyId); // 新しい返信IDを保存
    }
  };

  const username = comment.user?.username || "匿名ユーザー";
  const displayName = comment.user?.display_name || username;

  return (
    <div
      id={`comment-${comment.id}`}
      className={`${depth > 0 ? "ml-3 sm:ml-8 mt-4" : ""}`}
    >
      <Card className="hover:shadow-sm transition-shadow py-1">
        <CardHeader className="pb-3 px-3 sm:px-6 pt-3 sm:pt-6">
          <div className="flex items-start justify-between gap-2">
            <div className="grid grid-cols-[auto_1fr] items-start gap-2 sm:gap-3">
              <Link href={`/users/${comment.user.username}`}>
                <Avatar className="h-6 w-6 sm:h-8 sm:w-8 flex-shrink-0">
                  <AvatarFallback className="bg-blue-100 text-blue-600 text-xs sm:text-sm">
                    {username.charAt(0).toUpperCase() || (
                      <User className="h-3 w-3 sm:h-4 sm:w-4" />
                    )}
                  </AvatarFallback>
                </Avatar>
              </Link>
              <div className="min-w-0 flex-1">
                <div className="flex items-center space-x-2">
                  <Link
                    href={`/users/${comment.user.username}`}
                    className="font-medium text-xs sm:text-sm truncate hover:underline"
                  >
                    {displayName}
                  </Link>
                  <div className="flex items-center space-x-1 sm:space-x-2 text-xs text-muted-foreground">
                    <span>•</span>
                    <Clock className="h-3 w-3 flex-shrink-0" />
                    <span className="truncate">
                      {new Date(comment.created_at).toLocaleString("ja-JP", {
                        month: "short",
                        day: "numeric",
                        hour: "2-digit",
                        minute: "2-digit",
                      })}
                    </span>
                  </div>
                </div>
                {comment.user?.username &&
                  comment.user.username !== displayName && (
                    <Link
                      href={`/users/${comment.user.username}`}
                      className="text-xs text-muted-foreground mt-1 hover:underline"
                    >
                      @{comment.user.username}
                    </Link>
                  )}
              </div>
            </div>
            {user && comment.user && user.id === comment.user.id && (
              <div className="flex-shrink-0">
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6"
                  onClick={() => setShowDeleteDialog(true)}
                >
                  <Trash2 className="h-4 w-4 text-gray-500" />
                </Button>
                <Dialog
                  open={showDeleteDialog}
                  onOpenChange={setShowDeleteDialog}
                >
                  <DialogContent>
                    <DialogHeader>
                      <DialogTitle>コメントの削除</DialogTitle>
                      <DialogDescription>
                        このコメントを本当に削除してもよろしいですか？
                      </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                      <Button
                        variant="outline"
                        onClick={() => {
                          handleDeleteComment();
                          setShowDeleteDialog(false);
                        }}
                      >
                        削除
                      </Button>
                      <Button
                        variant="ghost"
                        onClick={() => setShowDeleteDialog(false)}
                      >
                        キャンセル
                      </Button>
                    </DialogFooter>
                  </DialogContent>
                </Dialog>
              </div>
            )}
          </div>
        </CardHeader>

        <CardContent className="pt-0 px-3 sm:px-6 pb-3 sm:pb-6">
          <div className="prose prose-sm prose-gray max-w-none">
            <p className="whitespace-pre-wrap text-sm leading-relaxed break-words">
              {comment.content.replace(/\n{3,}/g, "\n\n")}
            </p>
          </div>

          {/* 返信の表示 */}
          {comment.replies && comment.replies.length > 0 && (
            <div className="mt-4">
              {/* 返信の折りたたみボタン */}
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setShowReplies(!showReplies)}
                className="text-gray-500 hover:text-gray-700 mb-2 text-xs sm:text-sm p-1 sm:p-2"
              >
                <MessageSquare className="h-3 w-3 mr-1" />
                {showReplies
                  ? "返信を隠す"
                  : `${comment.replies.length}件の返信を表示`}
              </Button>

              {/* 返信コメント一覧 */}
              {showReplies && (
                <div className="space-y-2">
                  {comment.replies.map((reply) => (
                    <CommentItem
                      key={reply.id}
                      comment={reply}
                      threadId={threadId}
                      depth={depth + 1}
                      maxDepth={maxDepth}
                    />
                  ))}
                </div>
              )}
            </div>
          )}

          {/* 返信ボタン */}
          {isAuthenticated && depth < maxDepth && (
            <div className="mt-4 pt-2 border-t border-gray-100">
              <Button
                variant="ghost"
                size="sm"
                onClick={handleReplyClick}
                className="text-gray-500 hover:text-blue-600 text-xs sm:text-sm p-1 sm:p-2"
              >
                <Reply className="h-3 w-3 mr-1" />
                返信
              </Button>
            </div>
          )}

          {/* 返信フォーム */}
          {showReplyForm && (
            <div className="mt-4">
              <ReplyCommentForm
                threadId={threadId}
                parentId={comment.id}
                onCancel={handleCancelReply}
                onReplySuccess={handleReplySuccess}
                replyingToUsername={username}
              />
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

export function CommentList({
  threadId,
  commentsResponse,
  isLoading,
  error,
}: CommentListProps) {
  if (isLoading) {
    return (
      <div className="space-y-4">
        <h3 className="text-base sm:text-lg font-semibold px-1">
          コメントを読み込み中...
        </h3>
        {Array.from({ length: 3 }).map((_, i) => (
          <Card key={i}>
            <CardHeader className="px-3 sm:px-6 pt-3 sm:pt-6">
              <div className="flex items-center space-x-2 sm:space-x-3">
                <Skeleton className="h-6 w-6 sm:h-8 sm:w-8 rounded-full flex-shrink-0" />
                <div className="space-y-2 flex-1">
                  <Skeleton className="h-3 sm:h-4 w-[120px] sm:w-[150px]" />
                  <Skeleton className="h-2 sm:h-3 w-[80px] sm:w-[100px]" />
                </div>
              </div>
            </CardHeader>
            <CardContent className="px-3 sm:px-6 pb-3 sm:pb-6">
              <Skeleton className="h-3 sm:h-4 w-full" />
              <Skeleton className="h-3 sm:h-4 w-[70%] mt-2" />
              <Skeleton className="h-3 sm:h-4 w-[50%] mt-2" />
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <Card className="border-destructive">
        <CardHeader className="px-3 sm:px-6 pt-3 sm:pt-6">
          <CardTitle className="text-destructive text-base sm:text-lg">
            コメントの読み込みに失敗しました
          </CardTitle>
          <CardDescription className="text-sm">
            コメントの読み込み中にエラーが発生しました。もう一度お試しください。
          </CardDescription>
        </CardHeader>
        <CardContent className="px-3 sm:px-6 pb-3 sm:pb-6">
          <Button
            variant="outline"
            onClick={() => window.location.reload()}
            size="sm"
            className="text-sm"
          >
            <RefreshCw className="h-4 w-4 mr-2" />
            再試行
          </Button>
        </CardContent>
      </Card>
    );
  }

  // レスポンス構造に対応した柔軟なデータ取得
  const comments: CommentResponse[] = commentsResponse?.comments || [];
  const totalCount = commentsResponse?.total_count || comments.length;

  if (comments.length === 0) {
    return (
      <Card>
        <CardContent className="flex flex-col items-center justify-center py-12 sm:py-16 px-4">
          <MessageSquare className="h-10 w-10 sm:h-12 sm:w-12 text-muted-foreground mb-3 sm:mb-4" />
          <h3 className="text-base sm:text-lg font-semibold mb-2">
            まだコメントがありません
          </h3>
          <p className="text-muted-foreground text-center text-sm sm:text-base">
            このスレッドについて最初にコメントしてみませんか！
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      <h3 className="text-base sm:text-lg font-semibold px-1">
        {totalCount} 件のコメント
      </h3>

      <div className="space-y-4 sm:space-y-6">
        {comments.map((comment) => (
          <CommentItem
            key={comment.id}
            comment={comment}
            threadId={threadId}
            depth={0}
          />
        ))}
      </div>
    </div>
  );
}
