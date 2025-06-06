"use client";

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
import { useRequestPasswordReset } from "@/generated/api";
import { Alert, AlertDescription } from "@/components/ui/alert";

// フォームのバリデーションスキーマ
const formSchema = z.object({
  email: z.string().email({
    message: "有効なメールアドレスを入力してください",
  }),
});

type FormValues = z.infer<typeof formSchema>;

export default function ForgotPasswordPage() {
  const [success, setSuccess] = useState(false);

  // リクエスト送信用のミューテーション
  const { mutate: requestPasswordReset, isPending } = useRequestPasswordReset({
    mutation: {
      onSuccess: () => {
        setSuccess(true);
      },
    },
  });

  // フォームの初期化
  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      email: "",
    },
  });

  // フォーム送信処理
  const onSubmit = (values: FormValues) => {
    requestPasswordReset({ data: values });
  };

  return (
    <div className="min-h-screen flex justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8">
        <div className="bg-white p-8 rounded-lg shadow-md">
          <h1 className="mb-6 text-2xl font-bold text-center">
            パスワードをお忘れですか？
          </h1>

          {success ? (
            <div className="space-y-4">
              <Alert>
                <AlertDescription>
                  パスワードリセット用のメールを送信しました。メールに記載されたリンクからパスワードを再設定してください。
                </AlertDescription>
              </Alert>
              <Button asChild className="w-full">
                <Link href="/login">ログインページに戻る</Link>
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
                  name="email"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>メールアドレス</FormLabel>
                      <FormControl>
                        <Input
                          placeholder="your-email@example.com"
                          type="email"
                          {...field}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <Button type="submit" className="w-full" disabled={isPending}>
                  {isPending ? "送信中..." : "パスワードリセットリンクを送信"}
                </Button>

                <div className="text-center text-sm">
                  <Link href="/login" className="text-primary hover:underline">
                    ログインページに戻る
                  </Link>
                </div>
              </form>
            </Form>
          )}
        </div>
      </div>
    </div>
  );
}
