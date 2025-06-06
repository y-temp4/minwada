"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { useAuth } from "@/providers/auth-provider";
import { toast } from "sonner";
import {
  getGetCurrentUserQueryKey,
  useChangePassword,
  useDeleteUser,
  useUpdateProfile,
  useUpdateEmail,
  useResendVerification,
} from "@/generated/api";
import { usernameSchema } from "@/schemas/user";

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
import { Button } from "@/components/ui/button";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { ProtectedRoute } from "@/components/protected-route";
import { useQueryClient } from "@tanstack/react-query";

// プロフィール更新のバリデーションスキーマ
const profileFormSchema = z.object({
  username: usernameSchema.optional(),
  display_name: z
    .string()
    .max(30, {
      message: "表示名は30文字以下である必要があります",
    })
    .optional(),
});

// パスワード変更のバリデーションスキーマ
const passwordFormSchema = z
  .object({
    current_password: z.string().min(1, {
      message: "現在のパスワードを入力してください",
    }),
    new_password: z
      .string()
      .min(8, {
        message: "パスワードは8文字以上である必要があります",
      })
      .max(100, {
        message: "パスワードは100文字以下である必要があります",
      }),
    confirm_password: z.string().min(1, {
      message: "パスワードを確認してください",
    }),
  })
  .refine((data) => data.new_password === data.confirm_password, {
    message: "パスワードが一致しません",
    path: ["confirm_password"],
  });

// メールアドレス変更のバリデーションスキーマ
const emailFormSchema = z.object({
  email: z
    .string()
    .email({
      message: "有効なメールアドレスを入力してください",
    })
    .min(1, {
      message: "メールアドレスを入力してください",
    }),
});

type ProfileFormValues = z.infer<typeof profileFormSchema>;
type PasswordFormValues = z.infer<typeof passwordFormSchema>;
type EmailFormValues = z.infer<typeof emailFormSchema>;

export default function SettingsPage() {
  const { user, logout } = useAuth();
  const router = useRouter();
  const updateProfileMutation = useUpdateProfile();
  const changePasswordMutation = useChangePassword();
  const deleteUserMutation = useDeleteUser();
  const updateEmailMutation = useUpdateEmail();
  const resendVerificationMutation = useResendVerification();

  const [isUpdatingProfile, setIsUpdatingProfile] = useState(false);
  const [isChangingPassword, setIsChangingPassword] = useState(false);
  const [isChangingEmail, setIsChangingEmail] = useState(false);
  const [isResendingVerification, setIsResendingVerification] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [deleteConfirmPassword, setDeleteConfirmPassword] = useState("");
  const [deleteError, setDeleteError] = useState("");

  // プロフィール更新フォーム
  const profileForm = useForm<ProfileFormValues>({
    resolver: zodResolver(profileFormSchema),
    values: {
      username: user?.username || "",
      display_name: user?.display_name || "",
    },
  });

  // パスワード変更フォーム
  const passwordForm = useForm<PasswordFormValues>({
    resolver: zodResolver(passwordFormSchema),
    defaultValues: {
      current_password: "",
      new_password: "",
      confirm_password: "",
    },
  });

  // メールアドレス変更フォーム
  const emailForm = useForm<EmailFormValues>({
    resolver: zodResolver(emailFormSchema),
    defaultValues: {
      email: "",
    },
  });

  const queryClient = useQueryClient();

  // プロフィール更新の送信ハンドラー
  async function onProfileSubmit(data: ProfileFormValues) {
    if (!user) return;

    setIsUpdatingProfile(true);
    try {
      await updateProfileMutation.mutateAsync({
        data: {
          username: data.username ? data.username.toLowerCase() : undefined,
          display_name: data.display_name,
          avatar_url: undefined, // 必要に応じて追加
        },
      });

      queryClient.invalidateQueries({
        queryKey: [...getGetCurrentUserQueryKey()],
      });

      toast.success("プロフィールが更新されました");
    } catch (error) {
      console.error("Profile update failed:", error);
      toast.error("プロフィールの更新に失敗しました");
    } finally {
      setIsUpdatingProfile(false);
    }
  }

  // パスワード変更の送信ハンドラー
  async function onPasswordSubmit(data: PasswordFormValues) {
    if (!user) return;

    setIsChangingPassword(true);
    try {
      await changePasswordMutation.mutateAsync({
        data: {
          current_password: data.current_password,
          new_password: data.new_password,
        },
      });
      toast.success("パスワードが変更されました");
      passwordForm.reset();
    } catch (error) {
      console.error("Password change failed:", error);
      toast.error("パスワードの変更に失敗しました");
    } finally {
      setIsChangingPassword(false);
    }
  }

  // メールアドレス変更の送信ハンドラー
  async function onEmailSubmit(data: EmailFormValues) {
    if (!user) return;

    // 現在のメールアドレスと同じかチェック
    if (data.email.toLowerCase() === user.email.toLowerCase()) {
      emailForm.setError("email", {
        message:
          "新しいメールアドレスは現在のメールアドレスと異なる必要があります",
      });
      return;
    }

    setIsChangingEmail(true);
    try {
      await updateEmailMutation.mutateAsync({
        data: {
          email: data.email.toLowerCase(),
        },
      });

      queryClient.invalidateQueries({
        queryKey: [...getGetCurrentUserQueryKey()],
      });

      toast.success(
        "メールアドレスが更新されました。確認メールを送信しました。"
      );
      emailForm.reset();
    } catch (error) {
      console.error("Email update failed:", error);
      toast.error("メールアドレスの更新に失敗しました");
    } finally {
      setIsChangingEmail(false);
    }
  }

  // メール確認の再送信ハンドラー
  async function handleResendVerification() {
    if (!user) return;

    // 既に確認済みの場合は再送信しない
    if (user.email_verified) {
      toast.info("メールアドレスは既に確認済みです");
      return;
    }

    setIsResendingVerification(true);
    try {
      await resendVerificationMutation.mutateAsync();
      toast.success("確認メールを再送信しました");
    } catch (error) {
      console.error("Verification email resend failed:", error);
      toast.error("確認メールの再送信に失敗しました");
    } finally {
      setIsResendingVerification(false);
    }
  }

  // アカウント削除のハンドラー
  async function handleDeleteAccount() {
    if (!deleteConfirmPassword) {
      setDeleteError("パスワードを入力してください");
      return;
    }

    try {
      // ユーザー削除APIを呼び出す
      await deleteUserMutation.mutateAsync();
      toast.success("アカウントが削除されました");
      logout();
      router.push("/");
    } catch (error) {
      console.error("Account deletion failed:", error);
      setDeleteError(
        "アカウントの削除に失敗しました。もう一度お試しください。"
      );
    }
  }

  return (
    <ProtectedRoute>
      <div className="container max-w-4xl py-10 mx-auto">
        <h1 className="text-3xl font-bold mb-8">アカウント設定</h1>

        {/* プロフィール設定セクション */}
        <section className="mb-10">
          <Card>
            <CardHeader>
              <CardTitle>プロフィール設定</CardTitle>
              <CardDescription>プロフィール情報を更新します</CardDescription>
            </CardHeader>
            <CardContent>
              <Form {...profileForm}>
                <form
                  onSubmit={profileForm.handleSubmit(onProfileSubmit)}
                  className="space-y-6"
                >
                  <FormField
                    control={profileForm.control}
                    name="username"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>ユーザー名</FormLabel>
                        <FormControl>
                          <Input
                            placeholder="ユーザー名"
                            {...field}
                            value={field.value || ""}
                          />
                        </FormControl>
                        <FormDescription>
                          アプリ内でのあなたの識別子です
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <FormField
                    control={profileForm.control}
                    name="display_name"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>表示名</FormLabel>
                        <FormControl>
                          <Input
                            placeholder="表示名"
                            {...field}
                            value={field.value || ""}
                          />
                        </FormControl>
                        <FormDescription>
                          他のユーザーに表示される名前です
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <Button type="submit" disabled={isUpdatingProfile}>
                    {isUpdatingProfile ? "更新中..." : "プロフィールを更新"}
                  </Button>
                </form>
              </Form>
            </CardContent>
          </Card>
        </section>

        {/* パスワード変更セクション */}
        <section className="mb-10">
          <Card>
            <CardHeader>
              <CardTitle>パスワード変更</CardTitle>
              <CardDescription>
                アカウントのパスワードを変更します
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Form {...passwordForm}>
                <form
                  onSubmit={passwordForm.handleSubmit(onPasswordSubmit)}
                  className="space-y-6"
                >
                  <FormField
                    control={passwordForm.control}
                    name="current_password"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>現在のパスワード</FormLabel>
                        <FormControl>
                          <Input
                            type="password"
                            placeholder="現在のパスワード"
                            {...field}
                          />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <FormField
                    control={passwordForm.control}
                    name="new_password"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>新しいパスワード</FormLabel>
                        <FormControl>
                          <Input
                            type="password"
                            placeholder="新しいパスワード"
                            {...field}
                          />
                        </FormControl>
                        <FormDescription>
                          8文字以上の安全なパスワードを設定してください
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <FormField
                    control={passwordForm.control}
                    name="confirm_password"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>パスワードの確認</FormLabel>
                        <FormControl>
                          <Input
                            type="password"
                            placeholder="新しいパスワードを再入力"
                            {...field}
                          />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <Button type="submit" disabled={isChangingPassword}>
                    {isChangingPassword ? "変更中..." : "パスワードを変更"}
                  </Button>
                </form>
              </Form>
            </CardContent>
          </Card>
        </section>

        {/* メールアドレス変更セクション */}
        <section className="mb-10">
          <Card>
            <CardHeader>
              <CardTitle>メールアドレス変更</CardTitle>
              <CardDescription>
                アカウントのメールアドレスを変更します。新しいメールアドレスで確認が必要になります。
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Form {...emailForm}>
                <form
                  onSubmit={emailForm.handleSubmit(onEmailSubmit)}
                  className="space-y-6"
                >
                  <div className="mb-4">
                    <p className="text-sm text-gray-500">
                      現在のメールアドレス: {user?.email}
                    </p>
                    <p className="text-xs text-gray-500 mt-1">
                      {user?.email_verified
                        ? "メールアドレスは確認済みです"
                        : "メールアドレスは未確認です。メールボックスを確認してください"}
                    </p>
                    {!user?.email_verified && (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        className="mt-2"
                        onClick={handleResendVerification}
                        disabled={isResendingVerification}
                      >
                        {isResendingVerification
                          ? "送信中..."
                          : "確認メールを再送信"}
                      </Button>
                    )}
                  </div>
                  <FormField
                    control={emailForm.control}
                    name="email"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>新しいメールアドレス</FormLabel>
                        <FormControl>
                          <Input
                            type="email"
                            placeholder="新しいメールアドレス"
                            {...field}
                          />
                        </FormControl>
                        <FormDescription>
                          確認メールが新しいメールアドレスに送信されます
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <Button type="submit" disabled={isChangingEmail}>
                    {isChangingEmail ? "更新中..." : "メールアドレスを変更"}
                  </Button>
                </form>
              </Form>
            </CardContent>
          </Card>
        </section>

        {/* アカウント削除セクション */}
        <section>
          <Card className="border-red-200">
            <CardHeader>
              <CardTitle className="text-red-600">アカウント削除</CardTitle>
              <CardDescription>
                アカウントを完全に削除します。この操作は取り消せません。
              </CardDescription>
            </CardHeader>
            <CardContent>
              <AlertDialog
                open={isDeleteDialogOpen}
                onOpenChange={setIsDeleteDialogOpen}
              >
                <AlertDialogTrigger asChild>
                  <Button variant="destructive">アカウントを削除</Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>
                      本当にアカウントを削除しますか？
                    </AlertDialogTitle>
                    <AlertDialogDescription>
                      この操作は取り消せません。すべての投稿、コメント、アカウント情報が完全に削除されます。
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <div className="space-y-4 py-4">
                    <p className="text-sm font-medium">
                      確認のためパスワードを入力してください
                    </p>
                    <Input
                      type="password"
                      placeholder="パスワード"
                      value={deleteConfirmPassword}
                      onChange={(e) => setDeleteConfirmPassword(e.target.value)}
                    />
                    {deleteError && (
                      <p className="text-sm text-red-500">{deleteError}</p>
                    )}
                  </div>
                  <AlertDialogFooter>
                    <AlertDialogCancel
                      onClick={() => {
                        setDeleteConfirmPassword("");
                        setDeleteError("");
                      }}
                    >
                      キャンセル
                    </AlertDialogCancel>
                    <AlertDialogAction
                      onClick={(e) => {
                        e.preventDefault();
                        handleDeleteAccount();
                      }}
                      className="bg-red-600 hover:bg-red-700"
                    >
                      削除を確定
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            </CardContent>
          </Card>
        </section>
      </div>
    </ProtectedRoute>
  );
}
