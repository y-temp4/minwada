import Link from "next/link";
import React from "react";

export default function Footer() {
  return (
    <footer className="bg-white border-t border-gray-200 mt-12">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 flex flex-col md:flex-row items-center justify-between">
        <div className="flex items-center space-x-2">
          <span className="font-bold text-lg text-gray-900">
            <Link href="/">みんなの話題</Link>
          </span>
          <span className="text-gray-400 text-sm">
            © {new Date().getFullYear()}
          </span>
        </div>
        <div className="flex space-x-6 mt-4 md:mt-0">
          <a
            href="/privacy"
            className="text-gray-500 hover:text-gray-900 text-sm transition"
          >
            プライバシー
          </a>
          <a
            href="/terms"
            className="text-gray-500 hover:text-gray-900 text-sm transition"
          >
            利用規約
          </a>
          <a
            href="https://github.com/y-temp4/minwada"
            target="_blank"
            rel="noopener noreferrer"
            className="text-gray-500 hover:text-gray-900 text-sm transition"
          >
            GitHub
          </a>
        </div>
      </div>
    </footer>
  );
}
