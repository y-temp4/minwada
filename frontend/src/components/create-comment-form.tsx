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

  const createCommentMutation = useCreateComment();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!content.trim()) return;

    try {
      await createCommentMutation.mutateAsync({
        threadId: threadId,
        data: {
          content: content.trim(),
          parent_id: parentId,
        },
      });

      // Reset form
      setContent("");

      // Invalidate queries to refresh the comments
      queryClient.invalidateQueries({
        queryKey: [`/api/threads/${threadId}/comments`],
      });

      onSuccess?.();
    } catch (error) {
      console.error("Failed to create comment:", error);
    }
  };

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
          />
          <div className="flex justify-end space-x-2">
            {onCancel && (
              <Button type="button" variant="outline" onClick={onCancel}>
                キャンセル
              </Button>
            )}
            <Button
              type="submit"
              disabled={!content.trim() || createCommentMutation.isPending}
            >
              {createCommentMutation.isPending ? "投稿中..." : "投稿"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
