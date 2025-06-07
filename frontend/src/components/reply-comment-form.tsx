"use client";

import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { toast } from "sonner";
import { useQueryClient } from "@tanstack/react-query";

import { Button } from "@/components/ui/button";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormMessage,
} from "@/components/ui/form";
import { Textarea } from "@/components/ui/textarea";
import { MessageSquare, X } from "lucide-react";
import {
  useCreateComment,
  getGetCommentsQueryKey,
  getGetThreadQueryKey,
} from "@/generated/api";

const replyCommentSchema = z.object({
  content: z
    .string()
    .min(1, "返信内容を入力してください")
    .max(5000, "返信は5000文字以内で入力してください"),
});

type ReplyCommentForm = z.infer<typeof replyCommentSchema>;

interface ReplyCommentFormProps {
  threadId: string;
  parentId: string;
  onCancel: () => void;
  onReplySuccess?: (newReplyId?: string) => void;
  replyingToUsername?: string;
}

export function ReplyCommentForm({
  threadId,
  parentId,
  onCancel,
  onReplySuccess,
  replyingToUsername,
}: ReplyCommentFormProps) {
  const queryClient = useQueryClient();
  const form = useForm<ReplyCommentForm>({
    resolver: zodResolver(replyCommentSchema),
    defaultValues: {
      content: "",
    },
  });

  const createCommentMutation = useCreateComment({
    mutation: {
      onSuccess: (response) => {
        toast.success("返信を投稿しました！");
        form.reset();
        onCancel(); // 返信フォームを閉じる
        // スレッド詳細（コメント数）も更新
        queryClient.invalidateQueries({
          queryKey: getGetThreadQueryKey(threadId),
        });
        // コメント一覧を再取得
        queryClient.invalidateQueries({
          queryKey: getGetCommentsQueryKey(threadId),
        });
        if (onReplySuccess) {
          onReplySuccess(response.id);
        }
      },
      onError: () => {
        toast.error("返信の投稿に失敗しました");
      },
    },
  });

  const onSubmit = async (data: ReplyCommentForm) => {
    try {
      await createCommentMutation.mutateAsync({
        threadId: threadId,
        data: {
          content: data.content.trim(),
          parent_id: parentId,
        },
      });
    } catch (error) {
      console.error("Error submitting reply:", error);
      toast.error("返信の投稿に失敗しました");
    }
  };

  const isPending = createCommentMutation.isPending;

  return (
    <div className="bg-gray-50 p-4 rounded-lg border-l-4 border-blue-200">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center space-x-2">
          <MessageSquare className="h-4 w-4 text-blue-600" />
          <span className="text-sm font-medium text-gray-700">
            {replyingToUsername ? `${replyingToUsername} に返信` : "返信を作成"}
          </span>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={onCancel}
          disabled={isPending}
        >
          <X className="h-4 w-4" />
        </Button>
      </div>

      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-3">
          <FormField
            control={form.control}
            name="content"
            render={({ field }) => (
              <FormItem>
                <FormControl>
                  <Textarea
                    placeholder="返信内容を入力してください..."
                    className="min-h-[100px] resize-none"
                    {...field}
                    disabled={isPending}
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <div className="flex justify-end space-x-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={onCancel}
              disabled={isPending}
            >
              キャンセル
            </Button>
            <Button type="submit" size="sm" disabled={isPending}>
              <MessageSquare className="mr-1 h-3 w-3" />
              {isPending ? "投稿中..." : "返信を投稿"}
            </Button>
          </div>
        </form>
      </Form>
    </div>
  );
}
