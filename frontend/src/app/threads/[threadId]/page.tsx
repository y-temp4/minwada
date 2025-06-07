import { getThread } from "@/generated/api";
import { ThreadDetailPage } from "./_component";
import { notFound } from "next/navigation";
import { Metadata, ResolvingMetadata } from "next";

type Props = {
  params: Promise<{
    threadId: string;
  }>;
};

export async function generateMetadata(
  { params }: Props,
  parent: ResolvingMetadata
): Promise<Metadata> {
  try {
    const { threadId } = await params;

    // fetch data
    const thread = await getThread(threadId);

    // optionally access and extend (rather than replace) parent metadata
    const previousImages = (await parent).openGraph?.images || [];

    return {
      title: thread.title,
      description: thread.content,
      openGraph: {
        images: ["/some-specific-page-image.jpg", ...previousImages],
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
    return (
      <ThreadDetailPage threadId={threadId} initialData={initialUserData} />
    );
  } catch (error) {
    if (isApiError(error) && error.status === 404) {
      notFound();
    }
    throw error;
  }
}
