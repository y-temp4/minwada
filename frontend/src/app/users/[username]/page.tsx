"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { useAuth } from "@/providers/auth-provider";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { CalendarIcon, Clock, Edit } from "lucide-react";
import { format } from "date-fns";
import { ja } from "date-fns/locale";

export default function UserProfilePage() {
  const { username } = useParams<{ username: string }>();
  const { user: currentUser } = useAuth();
  const router = useRouter();
  const [isCurrentUser, setIsCurrentUser] = useState(false);
  const [userNotFound, setUserNotFound] = useState(false);
  const [user, setUser] = useState<{
    id: string;
    username: string;
    display_name?: string;
    created_at: string;
  } | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // この実装はサンプルです。実際のアプリでは、ユーザー情報を取得するAPIを呼び出す必要があります。
    // 例: API `/api/users/${username}` からユーザー情報を取得

    // ここでは、現在のユーザーが要求されたユーザー名と一致するか確認しています
    if (currentUser && currentUser.username === username) {
      setIsCurrentUser(true);
      // @ts-expect-error wip
      setUser(currentUser);
      setLoading(false);
    } else {
      // デモのために、現在のユーザーをユーザー情報として使用します
      // 実際のアプリでは、この部分をAPIコールに置き換えてください
      if (currentUser) {
        // ダミーデータ（実際のアプリでは適切なAPIコールに置き換えてください）
        // ユーザーが存在するという想定
        setUser({
          id: "dummy-id",
          username: username as string,
          display_name: `${username}のプロフィール`,
          created_at: new Date().toISOString(),
        });
        setLoading(false);
      } else {
        // ユーザーが見つからない場合
        setUserNotFound(true);
        setLoading(false);
      }
    }
  }, [username, currentUser]);

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
        <div className="flex items-start justify-between">
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
            <h3 className="text-lg font-medium">最近の投稿</h3>
            <div className="mt-4 text-gray-500 text-center py-8">
              <Clock className="h-12 w-12 mx-auto text-gray-300" />
              <p className="mt-2">まだ投稿がありません</p>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
