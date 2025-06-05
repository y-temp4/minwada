"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { useAuth } from "@/providers/auth-provider";
import { toast } from "sonner";
import { useUpdateProfile } from "@/generated/api";

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

// プロフィール更新のバリデーションスキーマ
const profileFormSchema = z.object({
  username: z
    .string()
    .min(3, {
      message: "ユーザー名は3文字以上である必要があります",
    })
    .max(50, {
      message: "ユーザー名は50文字以下である必要があります",
    })
    .optional(),
  display_name: z
    .string()
    .min(2, {
      message: "表示名は2文字以上である必要があります",
    })
    .max(30, {
      message: "表示名は30文字以下である必要があります",
    })
    .optional(),
  email: z.string().email({
    message: "有効なメールアドレスを入力してください",
  }),
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

type ProfileFormValues = z.infer<typeof profileFormSchema>;
type PasswordFormValues = z.infer<typeof passwordFormSchema>;

export default function SettingsPage() {
  const { user, logout } = useAuth();
  const router = useRouter();
  const updateProfileMutation = useUpdateProfile();

  const [isUpdatingProfile, setIsUpdatingProfile] = useState(false);
  const [isChangingPassword, setIsChangingPassword] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [deleteConfirmPassword, setDeleteConfirmPassword] = useState("");
  const [deleteError, setDeleteError] = useState("");

  // プロフィール更新フォーム
  const profileForm = useForm<ProfileFormValues>({
    resolver: zodResolver(profileFormSchema),
    defaultValues: {
      username: user?.username || "",
      display_name: user?.display_name || "",
      email: user?.email || "",
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

  // プロフィール更新の送信ハンドラー
  async function onProfileSubmit(data: ProfileFormValues) {
    if (!user) return;

    setIsUpdatingProfile(true);
    try {
      await updateProfileMutation.mutateAsync({
        data: {
          username: data.username,
          display_name: data.display_name,
          avatar_url: undefined, // 必要に応じて追加
        },
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
  async function onPasswordSubmit(_data: PasswordFormValues) {
    if (!user) return;

    setIsChangingPassword(true);
    try {
      // TODO: パスワード変更APIが実装されたら置き換える
      toast.success("パスワードが変更されました");
      passwordForm.reset();
    } catch (error) {
      console.error("Password change failed:", error);
      toast.error("パスワードの変更に失敗しました");
    } finally {
      setIsChangingPassword(false);
    }
  }

  // アカウント削除のハンドラー
  async function handleDeleteAccount() {
    if (!deleteConfirmPassword) {
      setDeleteError("パスワードを入力してください");
      return;
    }

    try {
      // TODO: アカウント削除APIが実装されたら置き換える
      // await deleteUserMutation.mutateAsync({
      //   data: {
      //     password: deleteConfirmPassword,
      //   },
      // });
      toast.success("アカウントが削除されました");
      logout();
      router.push("/");
    } catch (error) {
      console.error("Account deletion failed:", error);
      setDeleteError(
        "アカウントの削除に失敗しました。パスワードが正しいか確認してください。"
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
                  <FormField
                    control={profileForm.control}
                    name="email"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>メールアドレス</FormLabel>
                        <FormControl>
                          <Input placeholder="メールアドレス" {...field} />
                        </FormControl>
                        <FormDescription>
                          アカウント通知の送信先になります
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
