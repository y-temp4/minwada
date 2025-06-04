"use client";

import { useParams } from "next/navigation";
import Link from "next/link";
import { ArrowLeft, MessageSquare, User, Clock } from "lucide-react";

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
import { Separator } from "@/components/ui/separator";
import { useGetThread, useGetComments } from "@/generated/api";
import { CreateCommentForm } from "@/components/create-comment-form";
import { CommentList } from "@/components/comment-list";

export default function ThreadDetailPage() {
  const params = useParams();
  const threadId = params.id as string;

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

  if (threadLoading) {
    return (
      <div className="min-h-screen bg-gray-50">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
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
                  <Skeleton className="h-6 w-[300px]" />
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
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
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
            <div className="flex items-start justify-between">
              <div className="flex items-center space-x-4">
                <Avatar className="h-12 w-12">
                  <AvatarFallback className="bg-orange-100 text-orange-600">
                    {thread.user?.username?.charAt(0).toUpperCase() || (
                      <User className="h-6 w-6" />
                    )}
                  </AvatarFallback>
                </Avatar>
                <div>
                  <CardTitle className="text-2xl">{thread.title}</CardTitle>
                  <CardDescription className="flex items-center space-x-4 mt-2">
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
                <p className="whitespace-pre-wrap">{thread.content}</p>
              </div>
            </CardContent>
          )}
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
    </div>
  );
}
