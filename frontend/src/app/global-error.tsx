"use client"; // Error boundaries must be Client Components

import { AlertTriangle, Home } from "lucide-react";

export default function GlobalError({
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  error: _error, // Error boundary receives the error object
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    // global-error must include html and body tags
    <html lang="ja">
      <body className="bg-background text-foreground">
        <div className="flex flex-col items-center min-h-screen py-8 px-4 text-center">
          <div className="space-y-8 max-w-md w-full">
            <div className="space-y-2">
              <div className="mx-auto w-16 h-16 bg-destructive/10 rounded-full flex items-center justify-center">
                <AlertTriangle className="h-8 w-8 text-destructive" />
              </div>
              <h1 className="text-4xl font-bold tracking-tighter text-destructive mt-4">
                エラーが発生しました
              </h1>
              <div className="h-1 w-16 mx-auto bg-destructive/80 rounded-full"></div>
            </div>

            <div className="space-y-4">
              <p className="text-muted-foreground">
                予期せぬエラーが発生しました。しばらく経ってからもう一度お試しください。
              </p>
            </div>

            <div className="pt-4 flex flex-col sm:flex-row gap-3 justify-center">
              <button
                onClick={() => reset()}
                className="px-6 py-3 bg-primary text-primary-foreground rounded-md font-medium hover:bg-primary/90 transition-colors"
              >
                もう一度試す
              </button>
              {/* eslint-disable-next-line @next/next/no-html-link-for-pages */}
              <a
                href="/"
                className="px-6 py-3 bg-secondary text-secondary-foreground rounded-md font-medium hover:bg-secondary/90 transition-colors flex items-center justify-center gap-2"
              >
                <Home className="h-4 w-4" />
                ホームに戻る
              </a>
            </div>
          </div>
        </div>
      </body>
    </html>
  );
}
