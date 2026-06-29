Add-Type @"
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public class WinEnum {
  public delegate bool EnumProc(IntPtr hWnd, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc lpEnumFunc, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
  [DllImport("user32.dll")] public static extern IntPtr GetWindowLongPtr(IntPtr hWnd, int nIndex);
  [DllImport("user32.dll")] public static extern IntPtr GetWindow(IntPtr hWnd, uint uCmd);
  public struct RECT { public int Left, Top, Right, Bottom; }
  public static List<object[]> Results = new List<object[]>();
  public static bool Callback(IntPtr hWnd, IntPtr lParam) {
    if (!IsWindowVisible(hWnd)) return true;
    var title = new StringBuilder(512); GetWindowText(hWnd, title, 512);
    var cls = new StringBuilder(256); GetClassName(hWnd, cls, 256);
    RECT r; GetWindowRect(hWnd, out r);
    uint pid; GetWindowThreadProcessId(hWnd, out pid);
    long ex = GetWindowLongPtr(hWnd, -20).ToInt64();
    IntPtr owner = GetWindow(hWnd, 4);
    Results.Add(new object[]{ hWnd.ToInt64(), pid, cls.ToString(), title.ToString(), r.Left, r.Top, r.Right-r.Left, r.Bottom-r.Top, ex, owner.ToInt64() });
    return true;
  }
}
"@

[WinEnum]::EnumWindows([WinEnum+EnumProc]{ param($h,$l) [WinEnum]::Callback($h,$l) }, [IntPtr]::Zero)
$steamPids = @{}
Get-Process -Name steam,steamwebhelper -ErrorAction SilentlyContinue | ForEach-Object { $steamPids[$_.Id] = $_.ProcessName }

[WinEnum]::Results |
  Where-Object { $steamPids.ContainsKey([uint32]$_[1]) } |
  Sort-Object { $_[1] }, { $_[3] } |
  ForEach-Object {
    $ex = [uint64]$_[8]
    [PSCustomObject]@{
      Handle = ('0x{0:X}' -f [int64]$_[0])
      PID = $_[1]
      Process = $steamPids[[uint32]$_[1]]
      Class = $_[2]
      Title = $_[3]
      X = $_[4]; Y = $_[5]; W = $_[6]; H = $_[7]
      Area = ([int64]$_[6] * [int64]$_[7])
      ExStyle = ('0x{0:X}' -f $ex)
      ToolWindow = (($ex -band 0x80) -ne 0)
      AppWindow = (($ex -band 0x80000) -ne 0)
      Owner = if ([int64]$_[9] -eq 0) { '-' } else { ('0x{0:X}' -f [int64]$_[9]) }
    }
  } | Format-Table -AutoSize
