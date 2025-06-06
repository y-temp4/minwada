"use client";

import { notFound, useParams } from "next/navigation";
import { useEffect, useState } from "react";
import { getGetCurrentUserQueryKey, verifyEmail } from "@/generated/api";
import Link from "next/link";
import { useQueryClient } from "@tanstack/react-query";

export default function VerifyEmailPage() {
  const params = useParams();
  const token = params.token as string;
  const [verificationStatus, setVerificationStatus] = useState<
    "pending" | "success" | "error"
  >("pending");
  const [message, setMessage] = useState<string>("メールアドレスを検証中...");

  const queryClient = useQueryClient();

  useEffect(() => {
    if (!token) {
      notFound();
    }

    const verifyEmailProcess = async () => {
      try {
        const response = await verifyEmail(token);
        setVerificationStatus("success");
        setMessage(response.message);
        queryClient.invalidateQueries({
          queryKey: [...getGetCurrentUserQueryKey()],
        });
      } catch (error) {
        console.error("Email verification failed:", error);
        setVerificationStatus("error");
        setMessage(
          "メールアドレスの検証に失敗しました。トークンが無効か期限切れの可能性があります。"
        );
      }
    };

    verifyEmailProcess();
  }, [token]);

  return (
    <div className="flex min-h-screen flex-col items-center p-4">
      <div className="w-full max-w-md rounded-lg border border-gray-200 bg-white p-6 shadow-md">
        <h1 className="mb-4 text-center text-2xl font-bold text-gray-900">
          メールアドレス検証
        </h1>

        <div className="mb-4 rounded-md p-4 text-center">
          {verificationStatus === "pending" && (
            <div className="text-gray-600">
              <div className="mb-2 flex justify-center">
                <div className="h-8 w-8 animate-spin rounded-full border-2 border-gray-300 border-t-blue-600"></div>
              </div>
              <p>{message}</p>
            </div>
          )}

          {verificationStatus === "success" && (
            <div className="text-green-600">
              <svg
                className="mx-auto mb-2 h-12 w-12"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M5 13l4 4L19 7"
                ></path>
              </svg>
              <p className="font-medium">{message}</p>
              {/* <p className="mt-2">ログインしてサービスをご利用いただけます。</p> */}
            </div>
          )}

          {verificationStatus === "error" && (
            <div className="text-red-600">
              <svg
                className="mx-auto mb-2 h-12 w-12"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M6 18L18 6M6 6l12 12"
                ></path>
              </svg>
              <p className="font-medium">{message}</p>
              <p className="mt-2">
                ログイン後に検証メールの再送信をお試しください。
              </p>
            </div>
          )}
        </div>

        <div className="flex justify-center space-x-4">
          {/* <Link
            href="/login"
            className="rounded-md bg-blue-600 px-4 py-2 text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50"
          >
            ログイン
          </Link> */}
          <Link
            href="/"
            className="rounded-md bg-gray-200 px-4 py-2 text-gray-700 hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-opacity-50"
          >
            ホームに戻る
          </Link>
        </div>
      </div>
    </div>
  );
}
