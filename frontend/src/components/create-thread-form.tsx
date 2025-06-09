"use client";

import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { toast } from "sonner";
import { useQueryClient } from "@tanstack/react-query";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Plus, LogIn, AlertCircle } from "lucide-react";
import { useCreateThread, getGetThreadsQueryKey } from "@/generated/api";
import { useAuth } from "@/providers/auth-provider";
import { Skeleton } from "@/components/ui/skeleton";

const createThreadSchema = z.object({
  title: z
    .string()
    .min(1, "タイトルは必須です")
    .max(50, "タイトルは50文字以内で入力してください"),
  content: z
    .string()
    .max(1000, "内容は1000文字以内で入力してください")
    .optional(),
});

type CreateThreadForm = z.infer<typeof createThreadSchema>;

export function CreateThreadForm() {
  const { isAuthenticated, isEmailVerified, loading } = useAuth();
  const queryClient = useQueryClient();
  const form = useForm<CreateThreadForm>({
    resolver: zodResolver(createThreadSchema),
    defaultValues: {
      title: "",
      content: "",
    },
  });

  const createThreadMutation = useCreateThread({
    mutation: {
      onSuccess: () => {
        toast.success("スレッドを作成しました！");
        form.reset();
        // スレッド一覧を再取得 - 自動生成されたクエリキー関数を使用
        queryClient.invalidateQueries({
          queryKey: getGetThreadsQueryKey(),
        });
      },
      onError: (error) => {
        console.error("Error creating thread:", error);
        toast.error("スレッドの作成に失敗しました");
      },
    },
  });

  const onSubmit = (data: CreateThreadForm) => {
    if (!isAuthenticated) {
      toast.error("スレッドを作成するにはログインが必要です");
      return;
    }

    if (!isEmailVerified) {
      toast.error("スレッドを作成するにはメールアドレスの認証が必要です");
      return;
    }

    createThreadMutation.mutate({
      data: {
        title: data.title,
        content: data.content || "",
      },
    });
  };

  const isPending = createThreadMutation.isPending;

  // ログイン状態をチェック中の場合
  if (loading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center">
            <Plus className="h-5 w-5 mr-2" />
            新しいスレッドを作成
          </CardTitle>
          <Skeleton className="h-5 w-3/4 mt-1" />
        </CardHeader>
        <CardContent>
          <div className="space-y-6">
            {/* タイトルフィールドのスケルトン */}
            <div className="space-y-2">
              <Skeleton className="h-[14px] w-20" />
              <Skeleton className="h-9 w-full" />
              <Skeleton className="h-4 w-3/4" />
            </div>

            {/* 内容フィールドのスケルトン */}
            <div className="space-y-2">
              <Skeleton className="h-4 w-24" />
              <Skeleton className="h-[118px] w-full" />
              <Skeleton className="h-4 w-2/3" />
            </div>

            {/* ボタンのスケルトン */}
            <Skeleton className="h-10 w-full" />
          </div>
        </CardContent>
      </Card>
    );
  }

  // ログインしていない場合
  if (!isAuthenticated) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center">
            <Plus className="h-5 w-5 mr-2" />
            新しいスレッドを作成
          </CardTitle>
          <CardDescription>
            コミュニティと新しい会話を始めましょう
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="text-center py-8">
            <LogIn className="h-12 w-12 mx-auto text-gray-400 mb-4" />
            <h3 className="text-lg font-semibold mb-2">ログインが必要です</h3>
            <p className="text-gray-600 mb-4">
              新しいスレッドを作成するにはログインが必要です。
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
          <CardTitle className="flex items-center">
            <Plus className="h-5 w-5 mr-2" />
            新しいスレッドを作成
          </CardTitle>
          <CardDescription>
            コミュニティと新しい会話を始めましょう
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="text-center py-8">
            <AlertCircle className="h-12 w-12 mx-auto text-amber-500 mb-4" />
            <h3 className="text-lg font-semibold mb-2">
              メールアドレスの認証が必要です
            </h3>
            <p className="text-gray-600 mb-4">
              新しいスレッドを作成するにはメールアドレスの認証が必要です。
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
        <CardTitle className="flex items-center">
          <Plus className="h-5 w-5 mr-2" />
          新しいスレッドを作成
        </CardTitle>
        <CardDescription>
          コミュニティと新しい会話を始めましょう
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
            <FormField
              control={form.control}
              name="title"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>タイトル</FormLabel>
                  <FormControl>
                    <Input
                      className="md:text-md"
                      placeholder="どんな話題について話しますか？"
                      {...field}
                      disabled={isPending}
                    />
                  </FormControl>
                  <FormDescription>
                    スレッドの内容が分かりやすいタイトルを付けてください
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="content"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>内容（任意）</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="スレッドの詳細を説明してください。"
                      className="min-h-[120px]"
                      {...field}
                      disabled={isPending}
                    />
                  </FormControl>
                  <FormDescription>
                    追加の詳細や背景情報を提供してください
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <Button type="submit" disabled={isPending} className="w-full">
              <Plus className="mr-2 h-4 w-4" />
              {isPending ? "作成中..." : "スレッドを作成"}
            </Button>
          </form>
        </Form>
      </CardContent>
    </Card>
  );
}
