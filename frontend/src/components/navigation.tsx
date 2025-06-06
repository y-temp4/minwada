"use client";

import Link from "next/link";
import { useAuth } from "@/providers/auth-provider";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { User, Settings, AlertCircle } from "lucide-react";

export function Navigation() {
  const { user, isAuthenticated, isEmailVerified, logout, loading } = useAuth();

  const handleLogout = async () => {
    await logout();
  };

  return (
    <>
      <header className="bg-white border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <div className="flex items-center">
              <Link href="/" className="text-2xl font-bold text-orange-600">
                Reddit Clone
              </Link>
            </div>

            <nav className="flex items-center space-x-4">
              {isAuthenticated && user && !isEmailVerified && (
                <AlertCircle className="h-5 w-5 text-amber-500 mr-1" />
              )}
              {loading ? (
                <div className="h-8 w-8 animate-pulse bg-gray-200 rounded-full" />
              ) : isAuthenticated && user ? (
                <>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button
                        variant="ghost"
                        className="p-0 h-8 cursor-pointer"
                      >
                        <Avatar className="h-8 w-8">
                          <AvatarFallback className="bg-orange-100 text-orange-600">
                            {(user.display_name || user.username)
                              .charAt(0)
                              .toUpperCase()}
                          </AvatarFallback>
                        </Avatar>
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end" className="w-56">
                      <DropdownMenuItem asChild>
                        <Link
                          href={`/users/${user.username}`}
                          className="flex items-center"
                        >
                          <User className="mr-2 h-4 w-4" />
                          <span>プロフィール</span>
                        </Link>
                      </DropdownMenuItem>
                      <DropdownMenuItem asChild>
                        <Link href="/settings" className="flex items-center">
                          <Settings className="mr-2 h-4 w-4" />
                          <span>設定</span>
                        </Link>
                      </DropdownMenuItem>
                      <DropdownMenuSeparator />
                      <DropdownMenuItem
                        onClick={handleLogout}
                        className="text-red-500"
                      >
                        ログアウト
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </>
              ) : (
                <>
                  <Link
                    href="/login"
                    className="text-gray-600 hover:text-gray-900"
                  >
                    ログイン
                  </Link>
                  <Link
                    href="/register"
                    className="bg-orange-600 text-white px-4 py-2 rounded-md hover:bg-orange-700"
                  >
                    新規登録
                  </Link>
                </>
              )}
            </nav>
          </div>
        </div>
      </header>

      {isAuthenticated && user && !isEmailVerified && (
        <div className="bg-amber-50 border-t border-b border-amber-200 py-2">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex items-center">
            <AlertCircle className="h-5 w-5 text-amber-500 mr-2" />
            <p className="text-amber-700">
              メールアドレスが認証されていません。
              <Link
                href="/settings"
                className="underline ml-1 text-amber-800 hover:text-amber-900"
              >
                設定ページから認証を行ってください。
              </Link>
            </p>
          </div>
        </div>
      )}
    </>
  );
}
