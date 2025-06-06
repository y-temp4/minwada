"use client";

import {
  createContext,
  useContext,
  useEffect,
  useState,
  ReactNode,
} from "react";
import { useGetCurrentUser, useLogout, useRefreshToken } from "@/generated/api";

interface User {
  id: string;
  username: string;
  email: string;
  display_name?: string | null;
  created_at: string;
  updated_at: string;
  email_verified: boolean;
  email_verified_at?: string | null;
}

interface AuthContextType {
  user: User | null;
  loading: boolean;
  isAuthenticated: boolean;
  isEmailVerified: boolean;
  login: (accessToken: string, refreshToken: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshTokens: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  const isEmailVerified = user?.email_verified ?? true;

  const logoutMutation = useLogout();
  const refreshTokenMutation = useRefreshToken();

  // ユーザー情報取得のクエリ（認証済みの場合のみ実行）
  const {
    data: userData,
    error: userError,
    refetch: refetchUser,
  } = useGetCurrentUser({
    query: {
      enabled: isAuthenticated,
      retry: false,
    },
  });

  // トークンの存在確認とユーザー情報の初期化
  useEffect(() => {
    const initializeAuth = async () => {
      const accessToken = localStorage.getItem("access_token");
      const refreshToken = localStorage.getItem("refresh_token");

      if (accessToken && refreshToken) {
        setIsAuthenticated(true);
        try {
          // ユーザー情報を取得してみる
          await refetchUser();
        } catch {
          // トークンが無効な場合、リフレッシュを試行
          try {
            await refreshTokens();
          } catch {
            // リフレッシュも失敗した場合、ログアウト
            handleLogout();
          }
        }
      }
      setLoading(false);
    };

    initializeAuth();
  }, []);

  // ユーザーデータの更新
  useEffect(() => {
    if (userData) {
      setUser(userData);
    }
  }, [userData]);

  // ユーザー情報取得エラーの処理
  useEffect(() => {
    if (userError && isAuthenticated) {
      // 401エラーの場合、トークンリフレッシュを試行
      const errorStatus = userError?.status;
      if (errorStatus === 401) {
        refreshTokens().catch(() => {
          handleLogout();
        });
      }
    }
  }, [userError, isAuthenticated]);

  const login = async (accessToken: string, refreshToken: string) => {
    try {
      localStorage.setItem("access_token", accessToken);
      localStorage.setItem("refresh_token", refreshToken);

      // まず認証状態を更新してクエリを有効化
      setIsAuthenticated(true);

      // ユーザー情報を取得して完了を待つ
      const result = await refetchUser();

      // ユーザー情報の取得が成功した場合のみ、状態を更新
      if (result.data) {
        setUser(result.data);
      }
    } catch (error) {
      console.error("Failed to fetch user data after login:", error);
      // エラーが発生した場合は認証状態をリセット
      handleLogout();
      throw error;
    }
  };

  const logout = async () => {
    try {
      const refreshToken = localStorage.getItem("refresh_token");
      if (refreshToken) {
        await logoutMutation.mutateAsync({
          data: { refresh_token: refreshToken },
        });
      }
    } catch (error) {
      console.error("Logout API error:", error);
    } finally {
      handleLogout();
    }
  };

  const handleLogout = () => {
    localStorage.removeItem("access_token");
    localStorage.removeItem("refresh_token");
    setUser(null);
    setIsAuthenticated(false);
  };

  const refreshTokens = async () => {
    const refreshToken = localStorage.getItem("refresh_token");
    if (!refreshToken) {
      throw new Error("No refresh token available");
    }

    try {
      const result = await refreshTokenMutation.mutateAsync({
        data: { refresh_token: refreshToken },
      });

      localStorage.setItem("access_token", result.access_token);
      localStorage.setItem("refresh_token", result.refresh_token);
      setIsAuthenticated(true);
      await refetchUser();
    } catch (error) {
      console.error("Token refresh failed:", error);
      handleLogout();
      throw error;
    }
  };

  // 自動トークンリフレッシュ（10分ごと、アクセストークンの有効期限は15分）
  useEffect(() => {
    if (!isAuthenticated) return;

    const interval = setInterval(
      () => {
        refreshTokens().catch((error) => {
          console.error("Auto refresh failed:", error);
        });
      },
      10 * 60 * 1000 // 10分間隔（アクセストークンの有効期限15分より短く設定）
    );

    return () => clearInterval(interval);
  }, [isAuthenticated]);

  const value: AuthContextType = {
    user,
    loading,
    isAuthenticated,
    isEmailVerified,
    login,
    logout,
    refreshTokens,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
