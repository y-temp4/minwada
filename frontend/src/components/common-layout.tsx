import { Navigation } from "@/components/navigation";
import Footer from "@/components/footer";

export default function CommonLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <div className="flex flex-col bg-gray-50">
      <Navigation />
      <main className="flex-grow max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-8 pb-20 w-full min-h-screen">
        {children}
      </main>
      <Footer />
    </div>
  );
}
