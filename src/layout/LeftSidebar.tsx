import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { WorkspaceList } from '@/features/workspace/WorkspaceList';
import { NewWorkspaceButton } from '@/features/workspace/NewWorkspaceButton';
import { useUIStore } from '@/stores/uiStore';
import { useResizable } from '@/hooks/useResizable';
import { ResizeHandle } from './ResizeHandle';

export function LeftSidebar() {
  const leftSidebarWidth = useUIStore((s) => s.leftSidebarWidth);
  const setLeftSidebarWidth = useUIStore((s) => s.setLeftSidebarWidth);

  const { width, onMouseDown } = useResizable(
    leftSidebarWidth,
    setLeftSidebarWidth,
  );

  return (
    <>
      <aside
        className="flex shrink-0 flex-col bg-sidebar text-sidebar-foreground"
        style={{ width }}
      >
        <div className="flex items-center px-3 py-2">
          <NewWorkspaceButton />
        </div>
        <Separator />
        <ScrollArea className="flex-1">
          <WorkspaceList />
        </ScrollArea>
      </aside>
      <ResizeHandle side="left" onMouseDown={onMouseDown} />
    </>
  );
}