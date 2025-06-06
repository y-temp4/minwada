import { LoginForm } from "@/components/login-form";
import Link from "next/link";

export default function LoginPage() {
  return (
    <div className="min-h-screen flex justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8">
        <LoginForm />
        <div className="text-center">
          <p className="text-sm text-gray-600">
            アカウントをお持ちでない方は{" "}
            <Link
              href="/register"
              className="font-medium text-blue-600 hover:text-blue-500"
            >
              新規登録はこちら
            </Link>
          </p>
        </div>
      </div>
    </div>
  );
}
