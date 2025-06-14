import { getThread } from "@/generated/api";
import { ThreadDetailPage } from "./_component";
import { notFound } from "next/navigation";
import type { Metadata, ResolvingMetadata } from "next";

type Props = {
  params: Promise<{
    threadId: string;
  }>;
};

export async function generateMetadata(
  { params }: Props,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  _parent: ResolvingMetadata
): Promise<Metadata> {
  try {
    const { threadId } = await params;

    // fetch data
    const thread = await getThread(threadId);

    // OGP画像のURL
    const ogpImageUrl = `${process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:8000"}/api/threads/${threadId}/ogp.png`;

    // サイトのURL
    const siteUrl = process.env.NEXT_PUBLIC_APP_URL || "http://localhost:3000";
    const threadUrl = `${siteUrl}/threads/${threadId}`;

    return {
      title: thread.title,
      description: thread.content,
      openGraph: {
        title: thread.title,
        description: thread.content || "",
        url: threadUrl,
        siteName: "みんなの話題",
        images: [
          {
            url: ogpImageUrl,
            width: 1200,
            height: 630,
            alt: `${thread.title} - by @${thread.user.username}`,
          },
        ],
        locale: "ja_JP",
        type: "article",
      },
      twitter: {
        card: "summary_large_image",
        title: thread.title,
        description: thread.content || "minwadaのスレッド",
        images: [ogpImageUrl],
        creator: `@${thread.user.username}`,
      },
    };
  } catch {
    return {
      title: "エラーが発生しました",
      openGraph: {
        images: [],
      },
    };
  }
}

const isApiError = (error: unknown): error is { status: number } => {
  return (
    typeof error === "object" &&
    error !== null &&
    "status" in error &&
    typeof (error as { status: number }).status === "number"
  );
};

export default async function Page({ params }: Props) {
  const threadId = await params.then((p) => p.threadId);

  try {
    const initialUserData = await getThread(threadId);

    const siteUrl = process.env.NEXT_PUBLIC_APP_URL || "http://localhost:3000";
    const threadUrl = `${siteUrl}/threads/${threadId}`;

    const jsonLd = {
      "@context": "https://schema.org",
      "@type": "Article",
      headline: initialUserData.title,
      description: initialUserData.content || "",
      author: {
        "@type": "Person",
        name:
          initialUserData.user.display_name || initialUserData.user.username,
        url: `${siteUrl}/users/${initialUserData.user.username}`,
      },
      datePublished: initialUserData.created_at,
      dateModified: initialUserData.updated_at,
      url: threadUrl,
      publisher: {
        "@type": "Organization",
        name: "みんなの話題",
        url: siteUrl,
      },
      mainEntityOfPage: {
        "@type": "WebPage",
        "@id": threadUrl,
      },
    };

    return (
      <>
        <script
          type="application/ld+json"
          // biome-ignore lint/security/noDangerouslySetInnerHtml:
          dangerouslySetInnerHTML={{
            __html: JSON.stringify(jsonLd).replace(/</g, "\\u003c"),
          }}
        />
        <ThreadDetailPage threadId={threadId} initialData={initialUserData} />
      </>
    );
  } catch (error) {
    if (isApiError(error) && error.status === 404) {
      notFound();
    }
    throw error;
  }
}
