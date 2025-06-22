import com.android.build.gradle.tasks.MergeSourceSetFolders
import com.nishtahir.CargoBuildTask
import org.jetbrains.kotlin.util.capitalizeDecapitalize.capitalizeAsciiOnly
import org.jetbrains.kotlin.util.capitalizeDecapitalize.toLowerCaseAsciiOnly

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.rust.android)
}

android {
    namespace = "com.burakguner.myapp.shared"
    compileSdk = 35

    ndkVersion = "29.0.13599879"

    defaultConfig {
        minSdk = 24

        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions {
        jvmTarget = "11"
    }
    sourceSets {
        named("main") {
            java.srcDir("${projectDir}/../../shared_types/generated/java")
        }
    }
}

dependencies {
    implementation(libs.jna) {
        artifact {
            type = "aar"
        }
    }

    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.appcompat)
    implementation(libs.material)
}

cargo {
    module = "../.."
    libname = "shared"
    profile = "debug"
    targets = listOf("arm", "arm64", "x86", "x86_64")
    extraCargoBuildArguments = listOf("--package", "shared")
    cargoCommand = System.getProperty("user.home") + "/.nix-profile/bin/cargo"
    rustcCommand = System.getProperty("user.home") + "/.nix-profile/bin/rustc"
    pythonCommand = "python3"
}

afterEvaluate {
    // The `cargoBuild` task isn't available until after evaluation.
    android.libraryVariants.configureEach {
        var productFlavor = ""
        productFlavors.forEach {
            productFlavor += it.name.capitalizeAsciiOnly()
        }

        val buildType = this.buildType.name.capitalizeAsciiOnly()
        tasks.named("preBuild") {
            this.dependsOn(tasks.named("typesGen"), tasks.named("bindGen"))
        }

        tasks.named("generate${productFlavor}${buildType}Assets") {
            this.dependsOn(tasks.named("cargoBuild"))
        }

        // The below dependsOn is needed till https://github.com/mozilla/rust-android-gradle/issues/85 is resolved this fix was got from #118
        tasks.withType(CargoBuildTask::class.java).forEach { buildTask ->
            tasks.withType(MergeSourceSetFolders::class.java).configureEach {
                inputs.dir(layout.buildDirectory.dir("rustJniLibs").get().dir(buildTask.toolchain!!.folder))
                dependsOn(buildTask)
            }
        }
    }
}

// The below dependsOn is needed till https://github.com/mozilla/rust-android-gradle/issues/85 is resolved this fix was got from #118
tasks.matching() { it.name.matches("merge.*JniLibFolders".toRegex()) }.configureEach {
    inputs.dir(layout.buildDirectory.dir("rustJniLibs/android"))
    dependsOn("cargoBuild")
}

tasks.register<Exec>("bindGen") {
    setWorkingDir("../../")

    val outDir = "${projectDir}/../../shared_types/generated/java"
    if (System.getProperty("os.name").toLowerCaseAsciiOnly().contains("windows")) {
        commandLine("cmd", "/c",
            "cargo build -p shared && " + "target\\debug\\uniffi-bindgen generate shared\\src\\shared.udl " + "--language kotlin " + "--out-dir " + outDir.replace('/', '\\'))
    } else {
        commandLine("sh", "-c",
            """\
                cargo build -p shared && \
                target/debug/uniffi-bindgen generate shared/src/shared.udl \
                --language kotlin \
                --out-dir $outDir
                """)
    }
}

tasks.register<Exec>("typesGen") {
    setWorkingDir("../../")
    if (System.getProperty("os.name").toLowerCaseAsciiOnly().contains("windows")) {
        commandLine("cmd", "/c", "cargo build -p shared_types")
    } else {
        commandLine("sh", "-c", "cargo build -p shared_types")
    }
}