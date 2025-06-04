"use client";

import { ThreadList } from "@/components/thread-list";
import { CreateThreadForm } from "@/components/create-thread-form";
import { Navigation } from "@/components/navigation";
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
    <div className="min-h-screen bg-gray-50">
      <Navigation />

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
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
                <CardTitle>Reddit Clone へようこそ</CardTitle>
                <CardDescription>
                  Next.js と Rust で構築されたモダンな議論プラットフォーム
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div>
                    <h4 className="font-semibold">機能</h4>
                    <ul className="text-sm text-gray-600 mt-2 space-y-1">
                      <li>• スレッドの作成と議論</li>
                      <li>• ネストしたコメントシステム</li>
                      <li>• ユーザー認証</li>
                      <li>• リアルタイム更新</li>
                    </ul>
                  </div>
                  <Separator />
                  <div>
                    <h4 className="font-semibold">はじめに</h4>
                    <p className="text-sm text-gray-600 mt-2">
                      アカウントを登録して、スレッドの作成や議論への参加を始めましょう。
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
      </main>
    </div>
  );
}
