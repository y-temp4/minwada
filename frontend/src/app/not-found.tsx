import { Button } from "@/components/ui/button";
import Link from "next/link";
import { Home } from "lucide-react";

export default function Page() {
  return (
    <div className="flex flex-col items-center min-h-screen py-8 px-4 bg-background text-center">
      <div className="space-y-8 max-w-md w-full">
        <div className="space-y-2">
          <h1 className="text-9xl font-bold tracking-tighter text-primary">
            404
          </h1>
          <div className="h-1 w-16 mx-auto bg-primary/80 rounded-full"></div>
        </div>

        <div className="space-y-4">
          <h2 className="text-2xl font-semibold text-foreground">
            ページが見つかりませんでした
          </h2>
          <p className="text-muted-foreground">
            お探しのページは存在しないか、移動した可能性があります。URLが正しいかご確認ください。
          </p>
        </div>

        <div className="pt-4">
          <Link href="/" className="inline-block">
            <Button className="gap-2" size="lg">
              <Home className="h-4 w-4" />
              ホームに戻る
            </Button>
          </Link>
        </div>
      </div>
    </div>
  );
}
