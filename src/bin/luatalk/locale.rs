use clap::Command;

pub trait Localize {
    fn localize(self, lang: SupportedLang) -> Self;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SupportedLang {
    #[default]
    En,
    ZhHans,
}

impl From<&str> for SupportedLang {
    fn from(lang: &str) -> Self {
        let lower_lang = lang.to_lowercase();
        let prefix = if let Some(prefix) = lower_lang.get(..2) {
            prefix
        } else {
            return SupportedLang::default();
        };
        use SupportedLang::*;
        match prefix {
            "en" => En,
            "zh" => ZhHans,
            _ => SupportedLang::default(),
        }
    }
}

impl Localize for Command {
    fn localize(self, lang: SupportedLang) -> Self {
        use SupportedLang::*;
        match lang {
            En => self.localize_en(),
            ZhHans => self.localize_zh_hans(),
        }
    }
}

trait AppCommandExt {
    fn localize_en(self) -> Self;
    fn localize_zh_hans(self) -> Self;
}

impl AppCommandExt for Command {
    #[inline]
    fn localize_en(self) -> Self {
        self
    }

    #[inline]
    fn localize_zh_hans(self) -> Self {
        self.about("使用 Lua 文件构建类MMT文章")
            .long_about(
                "
使用 Lua 文件构建类MMT文章。

支持 Lua 5.5。",
            )
            .after_help(
                "使用子命令 `generate config-help` 获取高级配置说明。

访问 `https://crates.io/crates/luatalk-cli` 或仓库获取更多信息。",
            )
            .mut_subcommand("generate", |cmd| {
                cmd.about("生成有用的文件，运行时或硬编码在二进制中")
                    .mut_arg("output", |arg| arg.help("输出文件。默认为 stdout"))
                    .mut_subcommand("example", |cmd| {
                        cmd.about("生成示例输入 Lua 文件").mut_arg("lang", |arg| {
                            arg.help("示例输入 Lua 文件的语言。默认为程序自主选择，根据系统语言。")
                        })
                    })
                    .mut_subcommand("typst", |cmd| {
                        cmd.about("生成用于渲染文章的 Typst 文件")
                            .mut_arg("data", |arg| {
                                arg.help("文章数据的 JSON 格式文件路径，来自 `do <INPUT> json`。")
                            })
                            .typst_output_options_localize_zh_hans()
                    })
                    .mut_subcommand("completion", |cmd| {
                        cmd.about("为指定的 shell 生成 shell 补全脚本").after_help(
                            "例如，`source <(luatalk generate completion bash)` \
                                    让 bash 用户在当前会话中加载补全脚本。",
                        )
                    })
                    .mut_subcommand("asset", |cmd| {
                        cmd.about("生成有用的资源文件。您也可以从源代码中获取它们。")
                    })
                    .mut_subcommand("config-help", |cmd| cmd.about("生成配置帮助文件"))
                    .mut_subcommand("license", |cmd| cmd.about("显示此程序及其依赖项的许可信息"))
            })
            .mut_subcommand("do", |cmd| {
                cmd.about("处理输入的 LuaTalk 文章.")
                    .mut_arg("input", |arg| arg.help("输入文件。'-' 为 stdin"))
                    .mut_arg("concat_pages", |arg| arg.help("将所有页面合并为单个页面"))
                    .mut_arg("format", |arg| {
                        arg.help(
                            "输入文件格式。默认为根据文件扩展名推断；对于 stdin，默认为 'json'。",
                        )
                    })
                    .mut_subcommand("show", |cmd| {
                        cmd.about("以 `luatalk::Article` 的结构显示 LuaTalk 文章")
                            .mut_arg("output", |arg| arg.help("输出文件。默认为 stdout"))
                    })
                    .mut_subcommand("json", |cmd| {
                        cmd.about("以 JSON 格式输出 LuaTalk 文章")
                            .mut_arg("output", |arg| arg.help("输出文件。默认为 stdout"))
                    })
                    .mut_subcommand("typst", |cmd| {
                        cmd.about("输出用于渲染文章的 Typst 文件")
                            .long_about(
                                "同时输出 JSON 文件和用于渲染文章的 Typst 文件。

这个操作类似于 `do <INPUT> json -o '<STEM>.json'`
和 `generate typst [OPTIONS] '<STEM>.json' -o '<STEM>.typ'` 的组合。",
                            )
                            .mut_arg("stem", |arg| {
                                arg.help(
                                "不完整的输出路径，以文件主文件名（无扩展名）为结尾。默认为None",
                            )
                            .long_help(
                                "不完整的输出路径，以文件主文件名（无扩展名）为结尾。默认为None。

默认为 None，表示与输入文件名的主文件名部分相同。
例如 'article' 或 'dir/article'。",
                            )
                            })
                            .typst_output_options_localize_zh_hans()
                            .url_fetch_options_localize_zh_hans()
                    })
                    .mut_subcommand("typst-compile", |cmd| {
                        cmd.about(
                            "通过仅此一个命令，使用 typst-cli，一键将文章编译为支持的输出格式",
                        )
                        .long_about(
                            "通过仅此一个命令，使用 typst-cli，一键将文章编译为支持的输出格式。

请注意，typst-cli 支持某些选项的环境变量，您可以使用它们来配置一些高级 typst-cli 选项。
对于 typst-cli 里一些更高级的用法，或如 `typst watch` 这种其他有用功能，
请使用 `do <INPUT> typst` 并手动运行 typst-cli。",
                        )
                        .mut_arg("output", |arg| {
                            arg.help("输出文件。默认为 None").long_help(
                                "输出文件。默认为 None。

对于单个文件（如 PDF）：一个文件路径。
None 表示与输入文件名的主文件名部分相同的文件。

对于多个文件（如 PNG 图片）：一个目录路径，或带有占位符 `p` 的格式字符串，表示从 1 开始的页码。
例如 'article_{p}.png'。
None 表示与输入文件名的主文件名部分相同的目录。",
                            )
                        })
                        .mut_arg("format", |arg| {
                            arg.help("输出格式。默认为 None，表示从输出文件的扩展名推断。")
                        })
                        .typst_output_options_localize_zh_hans()
                        .url_fetch_options_localize_zh_hans()
                        .mut_arg("keep_temp", |arg| {
                            arg.help("保留在系统临时目录中创建的临时文件，如果要调试程序会有用")
                        })
                    })
                    .mut_subcommand("momotalk", |cmd| {
                        cmd.about(
                            "Momotalk 导出 JSON 格式，适用于 'https://github.com/U1805/momotalk'",
                        )
                        .mut_arg("output", |arg| {
                            arg.help("输出文件。默认为 None").long_help(
                                "输出文件。默认为 None。

对于单文件：一个文件路径，或 '-' 表示 stdout。
None 表示 stdout。

对于多文件：一个目录路径，或带有占位符 `p` 的格式字符串，表示从 1 开始的页码。
例如 'article_{p}.json'。
None 表示与输入文件名的主文件名部分相同的目录。",
                            )
                        })
                        .mut_arg("pl", |arg| arg.help("用于手动设置输出为单文件或多文件。"))
                    })
            })
    }
}

trait TypstOutputOptionsArgsExt {
    fn typst_output_options_localize_zh_hans(self) -> Self;
}

impl TypstOutputOptionsArgsExt for Command {
    #[inline]
    fn typst_output_options_localize_zh_hans(self) -> Self {
        self.mut_arg("font_size", |arg| arg.help("字体大小（pt）"))
            .mut_arg("width", |arg| arg.help("页面宽度（pt）"))
            .mut_arg("font_family", |arg| {
                arg.help("要使用的字体名称。 例：'Noto Sans' 或 'BlueakaBetaGBK'")
            })
            .mut_arg("length_factor", |arg| {
                arg.help("用于缩放页面中所有元素的长度因子")
            })
    }
}

trait UrlFetchOptionsArgsExt {
    fn url_fetch_options_localize_zh_hans(self) -> Self;
}

impl UrlFetchOptionsArgsExt for Command {
    #[inline]
    fn url_fetch_options_localize_zh_hans(self) -> Self {
        self.mut_arg("offline", |arg| {
            arg.help("启用此选项以不从 URL 下载任何图像")
        })
    }
}
