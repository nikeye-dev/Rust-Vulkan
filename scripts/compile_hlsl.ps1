$shader_path = '..\resources\shaders\'
$compiler_path = 'C:\VulkanSDK\1.3.290.0\Bin\dxc.exe'

Get-ChildItem -Path $shader_path -Filter *.hlsl -Recurse -File -Name| ForEach-Object {
    $file_path = [System.IO.Path]::GetFullPath($_)
    $filename = [System.IO.Path]::GetFileNameWithoutExtension($_)

    $shader_type = ''
    If($filename.EndsWith('vert')) {
        $shader_type = 'vs_6_0'
    }
    ElseIf ($filename.EndsWith('frag')) {
        $shader_type = 'ps_6_0'
    }
    Else {
        continue
    }

    $file_out = [IO.Path]::Combine($shader_path, 'compiled', $filename)
    $file_out = $file_out + '.spv'
    & $compiler_path $file_path -T $shader_type -E main -Fo $file_out -spirv
}