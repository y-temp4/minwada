"use client";

import { use } from "react";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { useResetPassword } from "@/generated/api";
import { Alert, AlertDescription } from "@/components/ui/alert";

// フォームのバリデーションスキーマ
const formSchema = z
  .object({
    new_password: z.string().min(8, {
      message: "パスワードは8文字以上である必要があります",
    }),
    confirm_password: z.string(),
  })
  .refine((data) => data.new_password === data.confirm_password, {
    message: "パスワードが一致しません",
    path: ["confirm_password"],
  });

type FormValues = z.infer<typeof formSchema>;

export default function ResetPasswordPage({
  params,
}: {
  params: Promise<{ token: string }>;
}) {
  const [success, setSuccess] = useState(false);
  const [tokenError, setTokenError] = useState(false);

  // paramsをuseでラップする
  const { token } = use(params);

  // リセット用のミューテーション
  const {
    mutate: resetPassword,
    isPending,
    isError,
    error,
  } = useResetPassword({
    mutation: {
      onSuccess: () => {
        setSuccess(true);
      },
      onError: (error) => {
        if (error.status === 404) {
          setTokenError(true);
        }
      },
    },
  });

  // フォームの初期化
  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      new_password: "",
      confirm_password: "",
    },
  });

  // フォーム送信処理
  const onSubmit = (values: FormValues) => {
    resetPassword({
      token,
      data: {
        new_password: values.new_password,
      },
    });
  };

  // トークンが無効な場合はエラーメッセージを表示
  if (tokenError) {
    return (
      <div className="min-h-screen flex justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
        <div className="max-w-md w-full space-y-8">
          <div className="bg-white p-8 rounded-lg shadow-md">
            <h1 className="mb-6 text-2xl font-bold text-center">
              パスワードリセット
            </h1>
            <Alert variant="destructive">
              <AlertDescription>
                無効または期限切れのリセットトークンです。再度パスワードリセットをリクエストしてください。
              </AlertDescription>
            </Alert>
            <Button asChild className="mt-4 w-full">
              <Link href="/forgot-password">
                パスワードリセットをリクエスト
              </Link>
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8">
        <div className="bg-white p-8 rounded-lg shadow-md">
          <h1 className="mb-6 text-2xl font-bold text-center">
            新しいパスワードの設定
          </h1>

          {success ? (
            <div className="space-y-4">
              <Alert>
                <AlertDescription>
                  パスワードが正常にリセットされました。新しいパスワードでログインしてください。
                </AlertDescription>
              </Alert>
              <Button asChild className="w-full">
                <Link href="/login">ログインページへ</Link>
              </Button>
            </div>
          ) : (
            <Form {...form}>
              <form
                onSubmit={form.handleSubmit(onSubmit)}
                className="space-y-6"
              >
                <FormField
                  control={form.control}
                  name="new_password"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>新しいパスワード</FormLabel>
                      <FormControl>
                        <Input type="password" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="confirm_password"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>パスワードの確認</FormLabel>
                      <FormControl>
                        <Input type="password" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                {isError && error.status !== 404 && (
                  <Alert variant="destructive">
                    <AlertDescription>
                      エラーが発生しました。もう一度お試しください。
                    </AlertDescription>
                  </Alert>
                )}

                <Button type="submit" className="w-full" disabled={isPending}>
                  {isPending ? "送信中..." : "パスワードを更新"}
                </Button>
              </form>
            </Form>
          )}
        </div>
      </div>
    </div>
  );
}
