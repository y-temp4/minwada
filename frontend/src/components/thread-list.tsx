"use client";

import { useState } from "react";
import { useGetThreads } from "@/generated/api";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import {
  RefreshCw,
  MessageSquare,
  User,
  ChevronLeft,
  ChevronRight,
  ArrowBigUp,
  Zap,
} from "lucide-react";
import { Skeleton } from "@/components/ui/skeleton";
import Link from "next/link";

export function ThreadList() {
  const [page, setPage] = useState(1);
  const [limit] = useState(10);
  const {
    data: threadsResponse,
    isLoading,
    error,
    refetch,
  } = useGetThreads({
    page,
    limit,
  });

  const handleRefresh = () => {
    refetch();
  };

  const handlePreviousPage = () => {
    if (page > 1) {
      setPage(page - 1);
    }
  };

  const handleNextPage = () => {
    // 型定義が不完全なので、any型でアクセス
    const paginationData = threadsResponse?.threads;
    if (paginationData?.total_pages && page < paginationData.total_pages) {
      setPage(page + 1);
    }
  };

  if (isLoading) {
    return (
      <div className="space-y-4">
        {Array.from({ length: 5 }).map((_, i) => (
          <Card key={i}>
            <CardHeader>
              <div className="flex items-center space-x-4">
                <Skeleton className="h-10 w-10 rounded-full" />
                <div className="space-y-2">
                  <Skeleton className="h-4 w-[250px]" />
                  <Skeleton className="h-4 w-[200px]" />
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-[80%] mt-2" />
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <Card className="border-destructive">
        <CardHeader>
          <CardTitle className="text-destructive">
            スレッドの読み込みエラー
          </CardTitle>
          <CardDescription>
            スレッドの読み込みに失敗しました。もう一度お試しください。
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button variant="outline" onClick={handleRefresh}>
            <RefreshCw className="h-4 w-4 mr-2" />
            再試行
          </Button>
        </CardContent>
      </Card>
    );
  }

  // レスポンス構造に対応した柔軟なデータ取得
  const threads = threadsResponse?.threads?.data ?? [];
  const paginationData = threadsResponse?.threads;

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div className="flex items-center space-x-2">
          <Zap className="h-7 w-7 text-primary" />
          <h1 className="text-3xl font-bold">最新のスレッド</h1>
        </div>
        <Button variant="outline" size="sm" onClick={handleRefresh}>
          <RefreshCw className="h-4 w-4 mr-2" />
          更新
        </Button>
      </div>

      <div className="grid gap-4">
        {threads.map((thread) => (
          <Card key={thread.id} className="hover:shadow-md transition-shadow">
            <CardHeader>
              <div className="flex items-start justify-between">
                <div className="flex items-start space-x-3">
                  <Avatar>
                    <AvatarFallback className="bg-orange-100 text-orange-600">
                      {thread.user?.username.charAt(0).toUpperCase() || (
                        <User className="h-4 w-4" />
                      )}
                    </AvatarFallback>
                  </Avatar>
                  <div>
                    <Link href={`/threads/${thread.id}`}>
                      <CardTitle className="text-lg hover:text-primary cursor-pointer hover:underline break-all">
                        {thread.title}
                      </CardTitle>
                    </Link>
                    <CardDescription>
                      投稿者: {thread.user.display_name || "匿名ユーザー"} •{" "}
                      {new Date(thread.created_at).toLocaleDateString("ja-JP")}
                    </CardDescription>
                  </div>
                </div>
                <div className="flex items-center gap-1.5">
                  <Badge variant="outline" className="font-medium">
                    <ArrowBigUp className="h-3.5 w-3.5 mr-1 text-primary" />
                    {thread.upvote_count - thread.downvote_count}
                  </Badge>
                  <Badge variant="secondary">
                    <MessageSquare className="h-3 w-3 mr-1" />
                    {thread.comment_count || 0}
                  </Badge>
                </div>
              </div>
            </CardHeader>
            {thread.content && (
              <CardContent>
                <p className="text-muted-foreground line-clamp-3 break-all">
                  {thread.content}
                </p>
              </CardContent>
            )}
          </Card>
        ))}
      </div>

      {/* ページネーション */}
      {paginationData?.total_pages && paginationData.total_pages > 1 && (
        <div className="flex items-center justify-between gap-1">
          <Button
            variant="outline"
            onClick={handlePreviousPage}
            disabled={page <= 1}
            className="flex items-center gap-1"
          >
            <ChevronLeft className="h-4 w-4" />
            <span className="hidden sm:inline-block">前のページ</span>
          </Button>

          <span className="text-sm text-gray-600">
            ページ {page} / {paginationData.total_pages}
            {paginationData.total && (
              <>
                {" "}
                ({paginationData.total} 件中{" "}
                {Math.min((page - 1) * limit + 1, paginationData.total)} -{" "}
                {Math.min(page * limit, paginationData.total)} 件表示)
              </>
            )}
          </span>

          <Button
            variant="outline"
            onClick={handleNextPage}
            disabled={
              !paginationData?.total_pages || page >= paginationData.total_pages
            }
            className="flex items-center gap-1"
          >
            <span className="hidden sm:inline-block">次のページ</span>
            <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
      )}

      {threads.length === 0 && (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-16">
            <MessageSquare className="h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-semibold mb-2">
              まだスレッドがありません
            </h3>
            <p className="text-muted-foreground text-center">
              最初の議論を始めてみませんか？
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
