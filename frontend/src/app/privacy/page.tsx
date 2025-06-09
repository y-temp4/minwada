export default function PrivacyPage() {
  return (
    <div className="max-w-2xl mx-auto bg-white rounded-lg shadow p-8">
      <h1 className="text-2xl font-bold mb-4">プライバシーポリシー</h1>
      <p className="mb-4">
        みんなの話題（以下「当サービス」）は、ユーザーのプライバシーを尊重し、個人情報の保護に努めます。
      </p>
      <h2 className="text-xl font-semibold mt-6 mb-2">1. 収集する情報</h2>
      <ul className="list-disc pl-6 mb-4 text-gray-700">
        <li>
          メールアドレス、ユーザー名など、アカウント作成時にご提供いただく情報
        </li>
        <li>投稿・コメントなど、サービス利用時に生成される情報</li>
        <li>サービス改善のためのアクセスログやCookie等の技術情報</li>
      </ul>
      <h2 className="text-xl font-semibold mt-6 mb-2">2. 利用目的</h2>
      <ul className="list-disc pl-6 mb-4 text-gray-700">
        <li>サービスの提供・運営・改善のため</li>
        <li>不正利用防止やお問い合わせ対応のため</li>
        <li>法令遵守のため</li>
      </ul>
      <h2 className="text-xl font-semibold mt-6 mb-2">3. 第三者提供</h2>
      <p className="mb-4">
        法令に基づく場合を除き、ユーザーの同意なく第三者に個人情報を提供することはありません。
      </p>
      <h2 className="text-xl font-semibold mt-6 mb-2">4. 外部サービス</h2>
      <p className="mb-4">
        当サービスでは、利便性向上や分析のために外部サービス（例：Google
        Analytics等）を利用する場合があります。これらのサービスで取得される情報については、各サービスのプライバシーポリシーをご確認ください。
      </p>
      <h2 className="text-xl font-semibold mt-6 mb-2">5. お問い合わせ</h2>
      <p className="mb-4">
        プライバシーに関するご質問やご要望は、
        <a
          href="https://forms.gle/s98g8WZHh4X1rpPD9"
          target="_blank"
          rel="noopener noreferrer"
          className="text-blue-500 hover:underline"
        >
          お問い合わせフォーム
        </a>
        よりご連絡ください。
      </p>
      <p className="text-sm text-gray-400 mt-8">2025年6月7日 制定</p>
    </div>
  );
}
