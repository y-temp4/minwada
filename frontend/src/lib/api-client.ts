import axios, { AxiosRequestConfig } from "axios";

// カスタムAxiosインスタンス
const apiClient = axios.create({
  baseURL: process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:8000",
  timeout: parseInt(process.env.NEXT_PUBLIC_API_TIMEOUT || "10000"),
  withCredentials: true, // JWT Cookieを送信するため
});

// リクエストインターセプター（JWTトークン付与）
apiClient.interceptors.request.use(
  (config) => {
    // localStorageからJWTトークンを取得してヘッダーに追加
    if (typeof window !== "undefined") {
      const token = localStorage.getItem("access_token");
      if (token) {
        config.headers.Authorization = `Bearer ${token}`;
      }
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// レスポンスインターセプター（トークンリフレッシュ処理）
apiClient.interceptors.response.use(
  (response) => response,
  async (error) => {
    if (error.response?.status === 401 && typeof window !== "undefined") {
      // 401エラーの場合、トークンリフレッシュを試行
      try {
        const baseURL =
          process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:8000";

        // リフレッシュトークンを取得
        const refreshToken = localStorage.getItem("refresh_token");
        if (!refreshToken) {
          throw new Error("リフレッシュトークンが存在しません");
        }

        const refreshResponse = await axios.post(
          `${baseURL}/api/auth/refresh`,
          { refresh_token: refreshToken },
          {
            withCredentials: true,
          }
        );

        const newToken = refreshResponse.data.access_token;
        // 新しいリフレッシュトークンも保存
        const newRefreshToken = refreshResponse.data.refresh_token;
        localStorage.setItem("access_token", newToken);
        localStorage.setItem("refresh_token", newRefreshToken);

        // 元のリクエストを再実行
        error.config.headers.Authorization = `Bearer ${newToken}`;
        return apiClient.request(error.config);
      } catch {
        // リフレッシュも失敗した場合はログインページにリダイレクト
        localStorage.removeItem("access_token");
        window.location.href = "/login";
      }
    }
    return Promise.reject(error);
  }
);

// Orval用のカスタムInstance関数
export const customInstance = async <T>(
  config: AxiosRequestConfig,
  options?: AxiosRequestConfig
): Promise<T> => {
  const { data } = await apiClient({
    ...config,
    ...options,
  });
  return data;
};

export default apiClient;
