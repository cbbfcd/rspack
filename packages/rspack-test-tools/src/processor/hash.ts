import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions
} from "../type";
import { type IMultiTaskProcessorOptions, MultiTaskProcessor } from "./multi";

export interface IHashProcessorOptions<T extends ECompilerType>
	extends Omit<IMultiTaskProcessorOptions<T>, "runable"> {}

const REG_ERROR_CASE = /error$/;

export class HashProcessor<
	T extends ECompilerType
> extends MultiTaskProcessor<T> {
	constructor(_hashOptions: IHashProcessorOptions<T>) {
		super({
			defaultOptions: HashProcessor.defaultOptions<T>,
			overrideOptions: HashProcessor.overrideOptions<T>,
			runable: false,
			..._hashOptions
		});
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const testConfig = context.getTestConfig();
		const stats = compiler.getStats();
		if (!stats) {
			env.expect(false);
			return;
		}
		if (REG_ERROR_CASE.test(this._options.name)) {
			env.expect(stats.hasErrors());
		} else {
			env.expect(!stats.hasErrors());
		}

		if (typeof testConfig.validate === "function") {
			testConfig.validate(stats);
		} else {
			throw new Error(
				"HashTestCases should have test.config.js and a validate method"
			);
		}
	}

	static defaultOptions<T extends ECompilerType>(
		index: number,
		context: ITestContext
	): TCompilerOptions<T> {
		return {
			context: context.getSource(),
			output: {
				path: context.getDist()
			},
			experiments: {
				css: true,
				rspackFuture: {
					bundlerInfo: {
						force: false
					}
				}
			}
		} as TCompilerOptions<T>;
	}
	static overrideOptions<T extends ECompilerType>(
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	): void {
		if (!options.entry) {
			options.entry = "./index.js";
		}
		if (!global.printLogger) {
			options.infrastructureLogging = {
				level: "error"
			};
		}
	}
}
