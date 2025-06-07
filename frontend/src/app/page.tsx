"use client";

import { ThreadList } from "@/components/thread-list";
import { CreateThreadForm } from "@/components/create-thread-form";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";

export default function Home() {
  return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
      {/* Main Content Area */}
      <div className="lg:col-span-2 space-y-6">
        <CreateThreadForm />
        <ThreadList />
      </div>

      {/* Sidebar */}
      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle>みんなの話題(α版) へようこそ</CardTitle>
            <CardDescription>
              Next.js と Rust で構築されたモダンな議論プラットフォーム
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div>
                <h4 className="font-semibold">機能</h4>
                <ul className="text-sm text-gray-600 mt-2 space-y-1">
                  <li>• スレッドの作成とコメント</li>
                  <li>• 木構造のコメントシステム</li>
                  <li>• ユーザー認証</li>
                  <li>• リアルタイム更新(予定)</li>
                </ul>
              </div>
              <Separator />
              <div>
                <h4 className="font-semibold">はじめに</h4>
                <p className="text-sm text-gray-600 mt-2">
                  アカウントを登録して、スレッドの作成や会話への参加を始めましょう。
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>コミュニティガイドライン</CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="text-sm text-gray-600 space-y-2">
              <li>• 他のユーザーを尊重しましょう</li>
              <li>• トピックから逸れないようにしましょう</li>
              <li>• スパムや自己宣伝は禁止です</li>
              <li>• 適切な言葉遣いを心がけましょう</li>
            </ul>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
