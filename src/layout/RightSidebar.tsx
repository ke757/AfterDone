import { useUIStore } from '@/stores/uiStore';
import { useResizable } from '@/hooks/useResizable';
import { ResizeHandle } from './ResizeHandle';

export function RightSidebar() {
  const rightSidebarWidth = useUIStore((s) => s.rightSidebarWidth);
  const setRightSidebarWidth = useUIStore((s) => s.setRightSidebarWidth);

  const { width, onMouseDown } = useResizable(
    rightSidebarWidth,
    setRightSidebarWidth,
  );

  return (
    <>
      <ResizeHandle side="right" onMouseDown={onMouseDown} />
      <aside
        className="flex shrink-0 flex-col border-l border-border bg-sidebar text-sidebar-foreground"
        style={{ width }}
      >
        <div className="flex items-center justify-center p-4 text-sm text-muted-foreground">
          右侧栏（暂无内容）
        </div>
      </aside>
    </>
  );
}