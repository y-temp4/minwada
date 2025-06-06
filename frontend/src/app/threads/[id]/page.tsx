"use client";

import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import { useState } from "react";
import {
  ArrowLeft,
  MessageSquare,
  User,
  Clock,
  Trash2,
  ThumbsUp,
  ThumbsDown,
} from "lucide-react";
import { toast } from "sonner";
import { useQueryClient } from "@tanstack/react-query";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Skeleton } from "@/components/ui/skeleton";
import { Separator } from "@/components/ui/separator";
import {
  useGetThread,
  useGetComments,
  useDeleteThread,
  useVoteThread,
  getGetThreadsQueryKey,
  getGetThreadQueryKey,
} from "@/generated/api";
import { CreateCommentForm } from "@/components/create-comment-form";
import { CommentList } from "@/components/comment-list";
import { useAuth } from "@/providers/auth-provider";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";

export default function ThreadDetailPage() {
  const params = useParams();
  const threadId = params.id as string;
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  const {
    data: thread,
    isLoading: threadLoading,
    error: threadError,
  } = useGetThread(threadId);

  const {
    data: commentsResponse,
    isLoading: commentsLoading,
    error: commentsError,
  } = useGetComments(threadId);

  const router = useRouter();
  const queryClient = useQueryClient();
  const { user } = useAuth();

  const { mutate: deleteThread } = useDeleteThread();
  const { mutate: voteThread, isPending: isVoting } = useVoteThread();

  const handleDeleteThread = () => {
    deleteThread(
      { id: threadId },
      {
        onSuccess: () => {
          // スレッド一覧のキャッシュを無効化
          queryClient.invalidateQueries({
            queryKey: [...getGetThreadsQueryKey()],
          });
          toast.success("スレッドが削除されました");
          router.push("/");
        },
        onError: () => {
          toast.error("スレッドの削除に失敗しました");
        },
      }
    );
  };

  // 投票ハンドラ
  const handleVote = (voteType: "upvote" | "downvote") => {
    if (!user) {
      toast.error("投票にはログインが必要です");
      return;
    }
    voteThread(
      { id: threadId, data: { vote_type: voteType } },
      {
        onSuccess: () => {
          queryClient.invalidateQueries({
            queryKey: [...getGetThreadQueryKey(threadId)],
          });
        },
        onError: () => {
          toast.error("投票に失敗しました");
        },
      }
    );
  };

  if (threadLoading) {
    return (
      <div className="min-h-screen bg-gray-50">
        {/* ナビゲーション */}
        <div className="mb-6">
          <Skeleton className="h-10 w-24" />
        </div>

        {/* スレッド詳細 */}
        <Card className="mb-8">
          <CardHeader>
            <div className="flex items-center space-x-4">
              <Skeleton className="h-12 w-12 rounded-full" />
              <div className="space-y-2">
                <Skeleton className="h-6 w-[240px]" />
                <Skeleton className="h-4 w-[200px]" />
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <Skeleton className="h-4 w-full" />
            <Skeleton className="h-4 w-[80%] mt-2" />
            <Skeleton className="h-4 w-[60%] mt-2" />
          </CardContent>
        </Card>

        {/* コメントフォーム */}
        <Card className="mb-8">
          <CardContent className="p-6">
            <Skeleton className="h-4 w-[150px] mb-4" />
            <Skeleton className="h-32 w-full mb-4" />
            <Skeleton className="h-10 w-[120px]" />
          </CardContent>
        </Card>

        {/* コメント一覧 */}
        <div className="space-y-4">
          {Array.from({ length: 3 }).map((_, i) => (
            <Card key={i}>
              <CardHeader>
                <div className="flex items-center space-x-3">
                  <Skeleton className="h-8 w-8 rounded-full" />
                  <Skeleton className="h-4 w-[150px]" />
                </div>
              </CardHeader>
              <CardContent>
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-[70%] mt-2" />
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    );
  }

  if (threadError) {
    return (
      <div className="min-h-screen bg-gray-50">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="mb-6">
            <Button variant="ghost" asChild>
              <Link href="/">
                <ArrowLeft className="h-4 w-4 mr-2" />
                ホームに戻る
              </Link>
            </Button>
          </div>

          <Card className="border-destructive">
            <CardHeader>
              <CardTitle className="text-destructive">
                スレッドが見つかりません
              </CardTitle>
              <CardDescription>
                お探しのスレッドは存在しないか、削除された可能性があります。
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Button asChild>
                <Link href="/">ホームに戻る</Link>
              </Button>
            </CardContent>
          </Card>
        </div>
      </div>
    );
  }

  if (!thread) {
    return null;
  }

  return (
    <div className="min-h-screen bg-gray-50">
      {/* ナビゲーション */}
      <div className="mb-6">
        <Button variant="ghost" asChild>
          <Link href="/">
            <ArrowLeft className="h-4 w-4 mr-2" />
            ホームに戻る
          </Link>
        </Button>
      </div>

      {/* スレッド詳細 */}
      <Card className="mb-8">
        <CardHeader>
          <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-4">
            <div className="flex flex-row sm:flex-row items-start sm:space-x-4 gap-4 flex-1 min-w-0">
              <Avatar className="h-12 w-12 flex-shrink-0">
                <AvatarFallback className="bg-orange-100 text-orange-600">
                  {thread.user?.username?.charAt(0).toUpperCase() || (
                    <User className="h-6 w-6" />
                  )}
                </AvatarFallback>
              </Avatar>
              <div className="min-w-0">
                <CardTitle className="text-2xl break-all whitespace-pre-line w-full min-w-0">
                  {thread.title}
                </CardTitle>
                <CardDescription className="flex flex-wrap items-center gap-x-4 gap-y-1 mt-2 text-sm">
                  <span>
                    投稿者: {thread.user?.username || "不明なユーザー"}
                  </span>
                  <span className="flex items-center">
                    <Clock className="h-3 w-3 mr-1" />
                    {new Date(thread.created_at).toLocaleString("ja-JP")}
                  </span>
                  <span className="flex items-center">
                    <MessageSquare className="h-3 w-3 mr-1" />
                    {thread.comment_count} 件のコメント
                  </span>
                </CardDescription>
              </div>
            </div>
          </div>
        </CardHeader>
        {thread.content && (
          <CardContent>
            <div className="prose prose-gray max-w-none">
              <p className="whitespace-pre-wrap break-all">
                {thread.content.replace(/\n{3,}/g, "\n\n")}
              </p>
            </div>
          </CardContent>
        )}
        <CardFooter className="flex items-center justify-between gap-4">
          {/* 投票UI */}
          <div className="flex flex-row items-center gap-1">
            <Button
              size="icon"
              variant="ghost"
              aria-label="upvote"
              disabled={isVoting || !user}
              onClick={() => handleVote("upvote")}
              className="text-green-600 hover:bg-green-50"
            >
              <ThumbsUp className="h-5 w-5" />
            </Button>
            <span className="font-bold text-lg mx-1 select-none min-w-[2rem] text-center">
              {thread.upvote_count - thread.downvote_count}
            </span>
            <Button
              size="icon"
              variant="ghost"
              aria-label="downvote"
              disabled={isVoting || !user}
              onClick={() => handleVote("downvote")}
              className="text-red-600 hover:bg-red-50"
            >
              <ThumbsDown className="h-5 w-5" />
            </Button>
          </div>
          {user && thread.user && user.id === thread.user.id && (
            <div className="flex-shrink-0">
              <Dialog
                open={deleteDialogOpen}
                onOpenChange={setDeleteDialogOpen}
              >
                <DialogTrigger asChild>
                  <Button variant="outline" size="icon" className="mr-2">
                    <Trash2 className="h-5 w-5 text-gray-500" />
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>スレッドの削除</DialogTitle>
                    <DialogDescription>
                      このスレッドを本当に削除してもよろしいですか？
                    </DialogDescription>
                  </DialogHeader>
                  <DialogFooter>
                    <Button
                      variant="outline"
                      onClick={() => {
                        handleDeleteThread();
                        setDeleteDialogOpen(false);
                      }}
                    >
                      削除
                    </Button>
                    <Button
                      variant="ghost"
                      onClick={() => setDeleteDialogOpen(false)}
                    >
                      キャンセル
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
            </div>
          )}
        </CardFooter>
      </Card>

      <Separator className="mb-8" />

      {/* コメントセクション */}
      <div className="space-y-8">
        <div>
          <h2 className="text-xl font-semibold mb-4">コメント</h2>

          {/* 新しいコメント作成フォーム */}
          <CreateCommentForm threadId={threadId} />
        </div>

        {/* コメント一覧 */}
        <CommentList
          threadId={threadId}
          commentsResponse={commentsResponse}
          isLoading={commentsLoading}
          error={commentsError}
        />
      </div>
    </div>
  );
}
