// src/components/CommonLayout.tsx
import { NavigationBar } from "./NavigationBar";
import { ModeToggle } from "./ModeToggle";

export function CommonLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen bg-background text-foreground">
      {/* Header Container: ModeToggle Left, Nav Centered */}
      <header className="relative flex items-center justify-center p-4 border-b">
        <div className="absolute right-4">
          <ModeToggle />
        </div>
        <NavigationBar />
      </header>

      {/* Main Content Area */}
      <main className="max-w-full p-6">
        {children}
      </main>
    </div>
  );
}