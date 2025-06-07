"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { useAuth } from "@/providers/auth-provider";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { CalendarIcon, Clock, Edit, MessageSquare } from "lucide-react";
import { format } from "date-fns";
import { ja } from "date-fns/locale";
import {
  useGetUserByUsername,
  useGetUserThreads,
  useGetUserComments,
} from "@/generated/api";
import Link from "next/link";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

export default function UserProfilePage() {
  const { username } = useParams<{ username: string }>();
  const { user: currentUser } = useAuth();
  const router = useRouter();
  const [isCurrentUser, setIsCurrentUser] = useState(false);

  // useGetUserByUsernameフックを使用してユーザー情報を取得
  const {
    data: user,
    isLoading: userLoading,
    isError,
    error,
  } = useGetUserByUsername(username);

  // エラーが発生した場合はコンソールに出力
  useEffect(() => {
    if (isError && error) {
      console.error("ユーザー情報の取得に失敗しました:", error);
    }
  }, [isError, error]);

  // 現在のユーザーかどうかをチェック
  useEffect(() => {
    if (currentUser && currentUser.username === username) {
      setIsCurrentUser(true);
    } else {
      setIsCurrentUser(false);
    }
  }, [username, currentUser]);

  // ユーザーのスレッド一覧を取得
  const { data: userThreads, isLoading: threadsLoading } = useGetUserThreads(
    user?.id || "",
    { limit: 10, offset: 0 },
    { query: { enabled: !!user?.id } }
  );

  // ユーザーのコメント一覧を取得
  const { data: userComments, isLoading: commentsLoading } = useGetUserComments(
    user?.id || "",
    { limit: 10, offset: 0 },
    { query: { enabled: !!user?.id } }
  );

  const loading = userLoading || threadsLoading || commentsLoading;
  const userNotFound = isError;

  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-orange-600"></div>
      </div>
    );
  }

  if (userNotFound) {
    return (
      <Card>
        <CardContent className="pt-6">
          <div className="text-center py-8">
            <h2 className="text-2xl font-bold text-gray-700">
              ユーザーが見つかりません
            </h2>
            <p className="text-gray-500 mt-2">
              指定されたユーザー名 ({username}) のユーザーは存在しません。
            </p>
            <Button
              className="mt-4"
              variant="outline"
              onClick={() => router.push("/")}
            >
              ホームに戻る
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="pb-4">
        <div className="flex items-start justify-between flex-wrap gap-4">
          <div className="flex items-center space-x-4">
            <Avatar className="h-20 w-20">
              <AvatarFallback className="bg-orange-100 text-orange-600 text-2xl">
                {(user?.display_name || user?.username)
                  ?.charAt(0)
                  .toUpperCase()}
              </AvatarFallback>
            </Avatar>
            <div>
              <CardTitle className="text-2xl">
                {user?.display_name || user?.username}
              </CardTitle>
              <p className="text-gray-500">@{user?.username}</p>
            </div>
          </div>
          {isCurrentUser && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => router.push("/settings")}
            >
              <Edit className="h-4 w-4 mr-2" />
              プロフィールを編集
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <div className="flex items-center text-sm text-gray-500">
            <CalendarIcon className="mr-2 h-4 w-4" />
            <span>
              {user?.created_at
                ? `${format(new Date(user.created_at), "yyyy年MM月dd日", { locale: ja })}に登録`
                : "登録日不明"}
            </span>
          </div>

          {/* ここに追加のプロフィール情報を表示できます */}

          <div className="mt-6 pt-6 border-t">
            <Tabs defaultValue="threads" className="w-full">
              <TabsList className="mb-4">
                <TabsTrigger value="threads">投稿したスレッド</TabsTrigger>
                <TabsTrigger value="comments">投稿したコメント</TabsTrigger>
              </TabsList>

              <TabsContent value="threads">
                <h3 className="text-lg font-medium">投稿したスレッド</h3>
                {threadsLoading ? (
                  <div className="flex justify-center items-center h-32">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-600"></div>
                  </div>
                ) : userThreads && userThreads.length > 0 ? (
                  <div className="mt-4 space-y-4">
                    {userThreads.map((thread) => (
                      <Link
                        href={`/threads/${thread.id}`}
                        key={thread.id}
                        className="block"
                      >
                        <div className="p-4 rounded-lg border hover:bg-gray-50 transition-colors">
                          <div className="flex justify-between items-start gap-1">
                            <h4 className="font-medium text-gray-900 break-all">
                              {thread.title}
                            </h4>
                            <div className="flex items-center text-sm text-gray-500">
                              <MessageSquare className="h-4 w-4 mr-1" />
                              <span>{thread.comment_count}</span>
                            </div>
                          </div>
                          {thread.content && (
                            <p className="mt-2 text-gray-600 line-clamp-3 break-all">
                              {thread.content}
                            </p>
                          )}
                          <div className="mt-2 text-xs text-gray-500">
                            {format(
                              new Date(thread.created_at),
                              "yyyy年MM月dd日 HH:mm",
                              { locale: ja }
                            )}
                          </div>
                        </div>
                      </Link>
                    ))}
                  </div>
                ) : (
                  <div className="mt-4 text-gray-500 text-center py-8">
                    <Clock className="h-12 w-12 mx-auto text-gray-300" />
                    <p className="mt-2">まだ投稿がありません</p>
                  </div>
                )}
              </TabsContent>

              <TabsContent value="comments">
                <h3 className="text-lg font-medium">投稿したコメント</h3>
                {commentsLoading ? (
                  <div className="flex justify-center items-center h-32">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-600"></div>
                  </div>
                ) : userComments && userComments.length > 0 ? (
                  <div className="mt-4 space-y-4">
                    {userComments.map((comment) => (
                      <Link
                        href={`/threads/${comment.thread_id}`}
                        key={comment.id}
                        className="block"
                      >
                        <div className="p-4 rounded-lg border hover:bg-gray-50 transition-colors break-all">
                          <h4 className="font-medium text-gray-900 mb-2">
                            {comment.thread_title}
                          </h4>
                          <p className="text-gray-600 line-clamp-3">
                            {comment.content}
                          </p>
                          <div className="mt-2 text-xs text-gray-500">
                            {format(
                              new Date(comment.created_at),
                              "yyyy年MM月dd日 HH:mm",
                              { locale: ja }
                            )}
                          </div>
                        </div>
                      </Link>
                    ))}
                  </div>
                ) : (
                  <div className="mt-4 text-gray-500 text-center py-8">
                    <MessageSquare className="h-12 w-12 mx-auto text-gray-300" />
                    <p className="mt-2">まだコメントがありません</p>
                  </div>
                )}
              </TabsContent>
            </Tabs>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
