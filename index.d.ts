/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface WindowBounds {
  title: string
  x: number
  y: number
  width: number
  height: number
}
export function getDesktopWindowInfo(ppid: number): DesktopWindowInfo
export type JsDesktopWindowInfo = DesktopWindowInfo
export class DesktopWindowInfo {
  winRects: Array<WindowBounds>
  constructor(winRects: Array<WindowBounds>)
  isOverlaps(x: number, y: number): WindowBounds | null
}
