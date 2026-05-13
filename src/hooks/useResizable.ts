import { useCallback, useRef, useState } from 'react';

// 拖动调整边栏宽度
export function useResizable(
  initialWidth: number,
  onResize: (width: number) => void,
  min = 160,
  max = 400,
  reverse = false,
) {
  const [width, setWidth] = useState(initialWidth);
  const isResizing = useRef(false);
  const startX = useRef(0);
  const startWidth = useRef(0);

  const onMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      isResizing.current = true;
      startX.current = e.clientX;
      startWidth.current = width;

      document.body.style.userSelect = 'none';
      document.body.style.cursor = 'col-resize';

      const onMouseMove = (ev: MouseEvent) => {
        if (!isResizing.current) return;
        const delta = reverse ? startX.current - ev.clientX : ev.clientX - startX.current;
        const newWidth = Math.min(max, Math.max(min, startWidth.current + delta));
        setWidth(newWidth);
        onResize(newWidth);
      };

      const onMouseUp = () => {
        isResizing.current = false;
        document.body.style.userSelect = '';
        document.body.style.cursor = '';
        document.removeEventListener('mousemove', onMouseMove);
        document.removeEventListener('mouseup', onMouseUp);
      };

      document.addEventListener('mousemove', onMouseMove);
      document.addEventListener('mouseup', onMouseUp);
    },
    [width, min, max, onResize],
  );

  return { width, onMouseDown };
}