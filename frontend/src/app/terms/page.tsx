export default function TermsPage() {
  return (
    <div className="max-w-2xl mx-auto bg-white rounded-lg shadow p-8">
      <h1 className="text-2xl font-bold mb-4">利用規約</h1>
      <p className="mb-4">
        この利用規約（以下「本規約」）は、みんなの話題（以下「当サービス」）の利用条件を定めるものです。ユーザーの皆さまには、本規約に同意の上、ご利用いただきます。
      </p>
      <h2 className="text-xl font-semibold mt-6 mb-2">1. サービスの内容</h2>
      <p className="mb-4">
        当サービスは、ユーザー同士が話題を投稿・共有できるコミュニティサービスです。運営者は、サービス内容の変更・停止・終了を予告なく行うことができます。
      </p>
      <h2 className="text-xl font-semibold mt-6 mb-2">2. 禁止事項</h2>
      <ul className="list-disc pl-6 mb-4 text-gray-700">
        <li>法令または公序良俗に反する行為</li>
        <li>他のユーザーや第三者の権利を侵害する行為</li>
        <li>スパム、誹謗中傷、差別的表現など不適切な投稿</li>
        <li>サービス運営を妨害する行為</li>
      </ul>
      <h2 className="text-xl font-semibold mt-6 mb-2">3. 免責事項</h2>
      <p className="mb-4">
        当サービスの利用により生じた損害について、運営者は一切の責任を負いません。ユーザー間のトラブルは当事者間で解決してください。
      </p>
      <h2 className="text-xl font-semibold mt-6 mb-2">4. 規約の変更</h2>
      <p className="mb-4">
        本規約は、必要に応じて予告なく変更されることがあります。変更後の規約は、当サービス上に掲載した時点で効力を生じます。
      </p>
      <h2 className="text-xl font-semibold mt-6 mb-2">5. 準拠法・裁判管轄</h2>
      <p className="mb-4">
        本規約は日本法に準拠し、当サービスに関する紛争は運営者の所在地を管轄する裁判所を第一審の専属的合意管轄裁判所とします。
      </p>
      <p className="text-sm text-gray-400 mt-8">2025年6月7日 制定</p>
    </div>
  );
}
