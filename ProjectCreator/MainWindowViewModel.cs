using System;
using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.Runtime.CompilerServices;
using System.Threading.Tasks;
using System.Windows.Input;
using Avalonia.Controls;
using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Threading;
using System.Text.RegularExpressions;

namespace ProjectCreator;

public class MainWindowViewModel : INotifyPropertyChanged
{
    private string _projectName = string.Empty;
    private bool _useVenv;
    private bool _installPackages;
    private bool _refreshTemplates;
    private bool _force;

    private string _pythonStatus = "Проверка...";
    private string _pipStatus = "Проверка...";
    private string _venvStatus = "Проверка...";
    private string _log = string.Empty;
    private bool _isBusy;
    private string _projectNameError = string.Empty;

    public event PropertyChangedEventHandler? PropertyChanged;

    public string ProjectName { get => _projectName; set { _projectName = value; OnPropertyChanged(); OnPropertyChanged(nameof(CanCreate)); } }
    public bool UseVenv { get => _useVenv; set { _useVenv = value; OnPropertyChanged(); } }
    public bool InstallPackages { get => _installPackages; set { _installPackages = value; OnPropertyChanged(); } }
    public bool RefreshTemplates { get => _refreshTemplates; set { _refreshTemplates = value; OnPropertyChanged(); } }
    public bool Force { get => _force; set { _force = value; OnPropertyChanged(); } }

    public string PythonStatus { get => _pythonStatus; private set { _pythonStatus = value; OnPropertyChanged(); OnPropertyChanged(nameof(CanCreate)); } }
    public string PipStatus { get => _pipStatus; private set { _pipStatus = value; OnPropertyChanged(); OnPropertyChanged(nameof(CanCreate)); } }
    public string VenvStatus { get => _venvStatus; private set { _venvStatus = value; OnPropertyChanged(); OnPropertyChanged(nameof(CanCreate)); } }
    public string Log { get => _log; private set { _log = value; OnPropertyChanged(); } }
    public bool IsBusy { get => _isBusy; private set { _isBusy = value; OnPropertyChanged(); OnPropertyChanged(nameof(CanCreate)); } }
    public string ProjectNameError { get => _projectNameError; private set { _projectNameError = value; OnPropertyChanged(); } }

    public bool CanCreate => !IsBusy
                             && !string.IsNullOrWhiteSpace(ProjectName)
                             && IsValidProjectName(ProjectName)
                             && PythonStatus.Contains("OK")
                             && PipStatus.Contains("OK")
                             && VenvStatus.Contains("OK");

    public ICommand CreateCommand => new AsyncCommand(CreateAsync);
    public ICommand BrowseCommand => new RelayCommand(Browse);
    public ICommand RefreshEnvCommand => new AsyncCommand(CheckEnvironmentAsync);

    public MainWindowViewModel()
    {
        _ = CheckEnvironmentAsync();
    }

    private async Task CheckEnvironmentAsync()
    {
        var pyOk = await CheckCommandAsync(new[] { "python", "--version" })
                  || await CheckCommandAsync(new[] { "python3", "--version" });
        PythonStatus = pyOk ? "Python: OK" : "Python: NOT FOUND (добавьте Python в PATH)";

        var pipOk = await CheckCommandAsync(new[] { "pip", "--version" })
                 || await CheckCommandAsync(new[] { "python", "-m", "pip", "--version" })
                 || await CheckCommandAsync(new[] { "python3", "-m", "pip", "--version" });
        PipStatus = pipOk ? "pip: OK" : "pip: NOT FOUND (установите pip)";

        // Принимаем либо наличие virtualenv, либо встроенного модуля venv
        var venvOk = await CheckCommandAsync(new[] { "virtualenv", "--version" })
                  || await CheckCommandAsync(new[] { "python", "-m", "virtualenv", "--version" })
                  || await CheckCommandAsync(new[] { "python3", "-m", "virtualenv", "--version" })
                  || await CheckCommandAsync(new[] { "python", "-c", "import venv; print('ok')" })
                  || await CheckCommandAsync(new[] { "python3", "-c", "import venv; print('ok')" });
        VenvStatus = venvOk ? "venv/virtualenv: OK" : "venv/virtualenv: NOT FOUND (установите virtualenv или используйте встроенный venv)";
    }

    private void Browse()
    {
        // зарезервировано для выбора папки, сейчас не требуется
    }

    private async Task CreateAsync()
    {
        IsBusy = true;
        var exe = FindPython();
        var scriptPath = Path.Combine(Environment.CurrentDirectory, "create_project.py");
        var start = new ProcessStartInfo
        {
            FileName = exe,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true,
            WorkingDirectory = Environment.CurrentDirectory
        };
        start.ArgumentList.Add(scriptPath);
        start.ArgumentList.Add(ProjectName);
        if (Force) start.ArgumentList.Add("--force");
        if (UseVenv) start.ArgumentList.Add("--venv");
        if (InstallPackages) start.ArgumentList.Add("--install");
        if (RefreshTemplates) start.ArgumentList.Add("--refresh-templates");

        AppendLog($"> {exe} create_project.py {ProjectName}{(Force ? " --force" : "")}{(UseVenv ? " --venv" : "")}{(InstallPackages ? " --install" : "")}{(RefreshTemplates ? " --refresh-templates" : "")}\n");

        // Показ модального окна ожидания
        var busy = new BusyWindow();
        var mainWindow = (Application.Current?.ApplicationLifetime as IClassicDesktopStyleApplicationLifetime)?.MainWindow;
        if (mainWindow != null)
        {
            // Показываем модальное окно, не ожидая его закрытия, чтобы не блокировать выполнение
            _ = busy.ShowDialog(mainWindow);
        }

        try
        {
            using var proc = Process.Start(start)!;
            var stdoutTask = Task.Run(async () => {
                string? line;
                while ((line = await proc.StandardOutput.ReadLineAsync()) != null)
                {
                    AppendLog(line + "\n");
                }
            });
            var stderrTask = Task.Run(async () => {
                string? line;
                while ((line = await proc.StandardError.ReadLineAsync()) != null)
                {
                    AppendLog(line + "\n");
                }
            });
            await Task.WhenAll(proc.WaitForExitAsync(), stdoutTask, stderrTask);
            AppendLog($"ExitCode: {proc.ExitCode}\n");
        }
        catch (Exception ex)
        {
            AppendLog("Ошибка запуска: " + ex.Message + "\n");
        }
        finally
        {
            if (busy.IsVisible) busy.Close();
            IsBusy = false;
        }
    }

    private static string FindPython()
    {
        // Windows/macOS/Linux: сначала python, потом python3 при запуске
        return "python";
    }

    private bool IsValidProjectName(string name)
    {
        // Разрешаем только ASCII буквы/цифры и символы - _ . ; первый символ — буква/цифра; длина <= 64
        var ok = Regex.IsMatch(name, "^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$");
        if (!ok)
        {
            ProjectNameError = "Имя: только латинские буквы/цифры и - _ . , без пробелов (до 64 символов).";
            return false;
        }

        // Нельзя заканчивать точкой или пробелом (Windows)
        if (name.EndsWith(".") || name.EndsWith(" "))
        {
            ProjectNameError = "Имя не должно заканчиваться точкой или пробелом.";
            return false;
        }

        // Запрещённые имена Windows
        var upper = name.ToUpperInvariant();
        string[] reserved = { "CON","PRN","AUX","NUL","COM1","COM2","COM3","COM4","COM5","COM6","COM7","COM8","COM9","LPT1","LPT2","LPT3","LPT4","LPT5","LPT6","LPT7","LPT8","LPT9" };
        foreach (var r in reserved)
        {
            if (upper == r)
            {
                ProjectNameError = "Недопустимое имя для Windows (зарезервировано).";
                return false;
            }
        }

        ProjectNameError = string.Empty;
        return true;
    }

    private async Task<bool> CheckCommandAsync(string[] cmd)
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = cmd[0],
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true
            };
            for (int i = 1; i < cmd.Length; i++) psi.ArgumentList.Add(cmd[i]);
            using var proc = Process.Start(psi)!;
            await proc.WaitForExitAsync();
            return proc.ExitCode == 0;
        }
        catch
        {
            return false;
        }
    }

    private void AppendLog(string text)
    {
        if (Dispatcher.UIThread.CheckAccess())
        {
            Log += text;
        }
        else
        {
            Dispatcher.UIThread.Post(() => Log += text);
        }
    }

    private void OnPropertyChanged([CallerMemberName] string? name = null)
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
    }
}

public class RelayCommand : ICommand
{
    private readonly Action _execute;
    private readonly Func<bool>? _canExecute;
    public RelayCommand(Action execute, Func<bool>? canExecute = null) { _execute = execute; _canExecute = canExecute; }
    public bool CanExecute(object? parameter) => _canExecute?.Invoke() ?? true;
    public void Execute(object? parameter) => _execute();
    public event EventHandler? CanExecuteChanged;
    public void RaiseCanExecuteChanged() => CanExecuteChanged?.Invoke(this, EventArgs.Empty);
}

public class AsyncCommand : ICommand
{
    private readonly Func<Task> _execute;
    private readonly Func<bool>? _canExecute;
    public AsyncCommand(Func<Task> execute, Func<bool>? canExecute = null) { _execute = execute; _canExecute = canExecute; }
    public bool CanExecute(object? parameter) => _canExecute?.Invoke() ?? true;
    public async void Execute(object? parameter) => await _execute();
    public event EventHandler? CanExecuteChanged;
    public void RaiseCanExecuteChanged() => CanExecuteChanged?.Invoke(this, EventArgs.Empty);
}


