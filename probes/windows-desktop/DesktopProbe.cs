using System;
using System.Drawing;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows.Forms;

// Isolated feasibility probe. No database, autostart, hooks, network, or global shortcuts.
// Only reparents its own window; never modifies desktop icon styles or restarts Explorer.
internal static class Native {
    internal delegate bool EnumProc(IntPtr h, IntPtr l);
    [DllImport("user32.dll")] internal static extern bool EnumWindows(EnumProc cb, IntPtr l);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] internal static extern IntPtr FindWindowEx(IntPtr p, IntPtr a, string c, string n);
    [DllImport("user32.dll", SetLastError=true)] internal static extern IntPtr SetParent(IntPtr h, IntPtr p);
    [DllImport("user32.dll")] internal static extern IntPtr GetParent(IntPtr h);
    [DllImport("user32.dll", EntryPoint="GetWindowLongW")] internal static extern int GetStyle(IntPtr h, int i);
    [DllImport("user32.dll", EntryPoint="SetWindowLongW")] internal static extern int SetStyle(IntPtr h, int i, int v);
    [DllImport("user32.dll")] internal static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int w, int z, uint flags);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] internal static extern int GetClassName(IntPtr h, StringBuilder s, int max);
    internal static string Class(IntPtr h) { var s=new StringBuilder(256); GetClassName(h,s,256); return s.ToString(); }
}

internal sealed class DesktopPanel : Form {
    private bool moving; private Point anchor;
    internal Action<string> Record;
    internal DesktopPanel(Action<string> record) {
        Record=record; Text="LittleTodo Desktop Panel Probe";
        FormBorderStyle=FormBorderStyle.None; ShowInTaskbar=false;
        Size=new Size(340,220); BackColor=Color.FromArgb(244,225,158);
        var title=new Label { Text="LittleTodo - drag this header", Dock=DockStyle.Top, Height=42, TextAlign=ContentAlignment.MiddleCenter };
        title.MouseDown+=(s,e)=>{if(e.Button==MouseButtons.Left){moving=true;anchor=e.Location;}};
        title.MouseMove+=(s,e)=>{if(moving){Location=new Point(Left+e.X-anchor.X,Top+e.Y-anchor.Y);}};
        title.MouseUp+=(s,e)=>{moving=false;Record("drag_end="+Location);};
        var check=new CheckBox { Text="P1  Synthetic task - click to complete", AutoSize=true, Location=new Point(18,65) };
        check.CheckedChanged+=(s,e)=>Record("checkbox="+check.Checked);
        var detail=new Button { Text="Open editable detail", Location=new Point(18,110), Size=new Size(220,34) };
        detail.Click+=(s,e)=>{Record("detail_clicked");var d=new Form {Text="LittleTodo Detail Probe",Size=new Size(400,240),StartPosition=FormStartPosition.CenterScreen};d.Controls.Add(new TextBox {Multiline=true,Dock=DockStyle.Fill,Text="Synthetic task only. Edit here."});d.Show();};
        Controls.Add(title);Controls.Add(check);Controls.Add(detail);
    }
}

internal sealed class Controller : Form {
    private DesktopPanel panel; private TextBox status;
    private string log; private bool attached; private int n;
    internal Controller(string logPath) {
        log=logPath; Directory.CreateDirectory(Path.GetDirectoryName(log)); File.WriteAllText(log,"");
        Text="LittleTodo Desktop Capability Probe"; Size=new Size(620,440); StartPosition=FormStartPosition.CenterScreen;
        status=new TextBox {Multiline=true,ReadOnly=true,Dock=DockStyle.Bottom,Height=255,ScrollBars=ScrollBars.Vertical};Controls.Add(status);
        var bar=new FlowLayoutPanel {Dock=DockStyle.Top,Height=110};Controls.Add(bar);
        Add(bar,"Attach to desktop",Attach);
        Add(bar,"Change opacity",()=>{panel.Opacity=panel.Opacity==1.0?0.78:1.0;Record("opacity="+panel.Opacity);});
        Add(bar,"Change background",()=>{panel.BackColor=panel.BackColor==Color.LightBlue?Color.FromArgb(244,225,158):Color.LightBlue;Record("background="+panel.BackColor);});
        Add(bar,"Move panel",()=>{panel.Location=new Point(panel.Left-25,panel.Top+15);Record("moved="+panel.Location);});
        Add(bar,"Read status",Sample);
        Add(bar,"Exit probe",()=>Close());
        Shown+=(s,e)=>{panel=new DesktopPanel(Record);panel.StartPosition=FormStartPosition.Manual;panel.Location=new Point(Screen.PrimaryScreen.WorkingArea.Right-380,90);panel.Show();Record("panel_created="+panel.Handle);Sample();};
        FormClosing+=(s,e)=>{if(panel!=null){panel.Dispose();}Record("probe_closed");};
        var timer=new Timer {Interval=1000};timer.Tick+=(s,e)=>{if(++n%10==0)Sample();};timer.Start();
    }
    private void Add(FlowLayoutPanel p,string label,Action action) {var b=new Button {Text=label,AutoSize=true};b.Click+=(s,e)=>{try{action();}catch(Exception ex){Record("ERROR="+ex.Message);}};p.Controls.Add(b);}
    private void Record(string text) {string line=DateTimeOffset.UtcNow.ToString("o")+" "+text;status.AppendText(line+Environment.NewLine);File.AppendAllText(log,line+Environment.NewLine);}
    private void Attach() {
        if(attached){Record("already_attached");return;}
        IntPtr host=IntPtr.Zero;
        Native.EnumWindows((h,l)=>{if(Native.FindWindowEx(h,IntPtr.Zero,"SHELLDLL_DefView",null)!=IntPtr.Zero){host=h;return false;}return true;},IntPtr.Zero);
        if(host==IntPtr.Zero){Record("FAIL=no_existing_desktop_host");return;}
        int style=Native.GetStyle(panel.Handle,-16);
        Native.SetStyle(panel.Handle,-16,(style&~unchecked((int)0x80000000))|0x40000000);
        Native.SetParent(panel.Handle,host);
        Native.SetWindowPos(panel.Handle,IntPtr.Zero,Screen.PrimaryScreen.WorkingArea.Right-380,90,340,220,0x0040|0x0020|0x0010);
        attached=Native.GetParent(panel.Handle)==host;
        Record("attached="+attached+" host_class="+Native.Class(host)+" host="+host);
        Sample();
    }
    private void Sample() {if(panel!=null)Record("parent="+Native.GetParent(panel.Handle)+" parent_class="+Native.Class(Native.GetParent(panel.Handle))+" visible="+panel.Visible+" bounds="+panel.Bounds+" opacity="+panel.Opacity);}
    [STAThread] private static void Main(string[] args) {Application.EnableVisualStyles();Application.Run(new Controller(args.Length>0?args[0]:Path.Combine(AppDomain.CurrentDomain.BaseDirectory,"probe.log")));}
}
