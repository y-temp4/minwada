import { RegisterForm } from "@/components/register-form";
import Link from "next/link";

export default function RegisterPage() {
  return (
    <div className="min-h-screen flex justify-center bg-gray-50 py-12 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8">
        <RegisterForm />
        <div className="text-center">
          <p className="text-sm text-gray-600">
            すでにアカウントをお持ちですか？{" "}
            <Link
              href="/login"
              className="font-medium text-blue-600 hover:text-blue-500"
            >
              ログインはこちら
            </Link>
          </p>
        </div>
      </div>
    </div>
  );
}
