"use client";

import { useRouter } from "next/navigation";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import * as z from "zod";
import { useRegister } from "@/generated/api";
import { useAuth } from "@/providers/auth-provider";
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
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { toast } from "sonner";

const registerSchema = z
  .object({
    username: z
      .string()
      .min(3, "ユーザー名は3文字以上で入力してください")
      .max(50, "ユーザー名は50文字以下で入力してください"),
    email: z.string().email("有効なメールアドレスを入力してください"),
    password: z
      .string()
      .min(8, "パスワードは8文字以上で入力してください")
      .max(100, "パスワードは100文字以下で入力してください"),
    confirmPassword: z
      .string()
      .min(8, "パスワード確認は8文字以上で入力してください"),
    displayName: z
      .string()
      .max(100, "表示名は100文字以下で入力してください")
      .optional(),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: "パスワードが一致しません",
    path: ["confirmPassword"],
  });

type RegisterFormValues = z.infer<typeof registerSchema>;

export function RegisterForm() {
  const router = useRouter();
  const { login } = useAuth();

  const form = useForm<RegisterFormValues>({
    resolver: zodResolver(registerSchema),
    defaultValues: {
      username: "",
      email: "",
      password: "",
      confirmPassword: "",
      displayName: "",
    },
  });

  const registerMutation = useRegister();

  const {
    formState: { isSubmitting, isSubmitSuccessful },
  } = form;

  const onSubmit = async (values: RegisterFormValues) => {
    await registerMutation.mutateAsync(
      {
        data: {
          username: values.username,
          email: values.email,
          password: values.password,
          display_name: values.displayName || null,
        },
      },
      {
        async onSuccess(data) {
          await login(data.access_token, data.refresh_token);
          toast.success("アカウントが作成されました！");
          router.push("/");
        },
        onError(error) {
          console.error("Register error:", error);
          if (error.status === 409) {
            toast.error(
              "このユーザー名またはメールアドレスは既に使用されています。"
            );
          } else {
            toast.error(
              "アカウントの作成に失敗しました。もう一度お試しください。"
            );
          }
        },
      }
    );
  };

  return (
    <Card className="w-full max-w-md mx-auto">
      <CardHeader className="space-y-1">
        <CardTitle className="text-2xl text-center">アカウント作成</CardTitle>
        <CardDescription className="text-center">
          新しいアカウントを作成してください
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="username"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>ユーザー名</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="username"
                      {...field}
                      disabled={isSubmitting}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="email"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>メールアドレス</FormLabel>
                  <FormControl>
                    <Input
                      type="email"
                      placeholder="your@email.com"
                      {...field}
                      disabled={isSubmitting}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="displayName"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>表示名（任意）</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="表示名"
                      {...field}
                      disabled={isSubmitting}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="password"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>パスワード</FormLabel>
                  <FormControl>
                    <Input
                      type="password"
                      placeholder="パスワード"
                      {...field}
                      disabled={isSubmitting}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="confirmPassword"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>パスワード確認</FormLabel>
                  <FormControl>
                    <Input
                      type="password"
                      placeholder="パスワードを再入力"
                      {...field}
                      disabled={isSubmitting}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <Button
              type="submit"
              className="w-full"
              disabled={isSubmitting || isSubmitSuccessful}
            >
              {isSubmitting ? "作成中..." : "アカウント作成"}
            </Button>
          </form>
        </Form>
      </CardContent>
    </Card>
  );
}
