"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { useCreateComment } from "@/generated/api";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import Link from "next/link";
import { LogIn, AlertCircle } from "lucide-react";
import { useAuth } from "@/providers/auth-provider";

interface CreateCommentFormProps {
  threadId: string;
  parentId?: string;
  onCancel?: () => void;
  onSuccess?: () => void;
}

export function CreateCommentForm({
  threadId,
  parentId,
  onCancel,
  onSuccess,
}: CreateCommentFormProps) {
  const [content, setContent] = useState("");
  const queryClient = useQueryClient();
  const { isAuthenticated, isEmailVerified, loading } = useAuth();

  const createCommentMutation = useCreateComment({
    mutation: {
      onSuccess: () => {
        toast.success("コメントを投稿しました！");
        setContent("");
        // コメント一覧を再取得
        queryClient.invalidateQueries({
          queryKey: [`/api/threads/${threadId}/comments`],
        });
        onSuccess?.();
      },
      onError: (error) => {
        console.error("Error creating comment:", error);
        toast.error("コメントの投稿に失敗しました");
      },
    },
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!content.trim()) return;

    if (!isAuthenticated) {
      toast.error("コメントを投稿するにはログインが必要です");
      return;
    }

    if (!isEmailVerified) {
      toast.error("コメントを投稿するにはメールアドレスの認証が必要です");
      return;
    }

    createCommentMutation.mutate({
      threadId: threadId,
      data: {
        content: content.trim(),
        parent_id: parentId,
      },
    });
  };

  const isPending = createCommentMutation.isPending;

  // ログイン状態をチェック中の場合
  if (loading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>{parentId ? "返信を投稿" : "コメントを投稿"}</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center py-8">読み込み中...</div>
        </CardContent>
      </Card>
    );
  }

  // ログインしていない場合
  if (!isAuthenticated) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>{parentId ? "返信を投稿" : "コメントを投稿"}</CardTitle>
          <CardDescription>
            コミュニティへの参加には登録が必要です
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="text-center py-8">
            <LogIn className="h-12 w-12 mx-auto text-gray-400 mb-4" />
            <h3 className="text-lg font-semibold mb-2">ログインが必要です</h3>
            <p className="text-gray-600 mb-4">
              コメントを投稿するにはログインが必要です。
            </p>
            <div className="space-x-2">
              <Button asChild>
                <Link href="/login">ログイン</Link>
              </Button>
              <Button variant="outline" asChild>
                <Link href="/register">新規登録</Link>
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  // メールアドレス認証が完了していない場合
  if (!isEmailVerified) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>{parentId ? "返信を投稿" : "コメントを投稿"}</CardTitle>
          <CardDescription>
            コミュニティへの参加には認証が必要です
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="text-center py-8">
            <AlertCircle className="h-12 w-12 mx-auto text-amber-500 mb-4" />
            <h3 className="text-lg font-semibold mb-2">
              メールアドレスの認証が必要です
            </h3>
            <p className="text-gray-600 mb-4">
              コメントを投稿するにはメールアドレスの認証が必要です。
            </p>
            <div className="space-x-2">
              <Button asChild>
                <Link href="/settings">認証ページへ移動</Link>
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>{parentId ? "返信を投稿" : "コメントを投稿"}</CardTitle>
        <CardDescription>
          {parentId
            ? "このコメントに返信してください"
            : "このスレッドについてのコメントを投稿してください"}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          <Textarea
            placeholder="コメントを入力してください..."
            value={content}
            onChange={(e) => setContent(e.target.value)}
            rows={4}
            required
            disabled={isPending}
          />
          <div className="flex justify-end space-x-2">
            {onCancel && (
              <Button
                type="button"
                variant="outline"
                onClick={onCancel}
                disabled={isPending}
              >
                キャンセル
              </Button>
            )}
            <Button type="submit" disabled={!content.trim() || isPending}>
              {isPending ? "投稿中..." : "投稿"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
