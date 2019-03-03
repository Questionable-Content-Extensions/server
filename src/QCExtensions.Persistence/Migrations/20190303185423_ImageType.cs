using Microsoft.EntityFrameworkCore.Migrations;

namespace QCExtensions.Persistence.Migrations
{
	public partial class ImageType : Migration
	{
		protected override void Up(MigrationBuilder migrationBuilder)
		{
			migrationBuilder.AddColumn<int>(
				name: "ImageType",
				table: "comic",
				nullable: false,
				defaultValue: 0);

			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 158");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 194");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 504");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 1719");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 1950");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 1975");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 1988");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2002");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2018");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2019");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2020");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2047");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2127");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2162");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2181");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2182");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2211");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2228");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2229");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2231");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2234");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2235");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2250");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2261");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2282");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2292");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2296");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2297");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2311");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2313");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2314");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2316");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2326");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2337");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2350");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2359");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2361");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2420");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2459");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2470");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2483");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2493");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2499");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2513");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2517");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2518");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2519");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2574");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2589");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2893");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2900");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2939");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2953");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2969");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 2970");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2981");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2983");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2985");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 2986");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3219");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3220");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3251");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3255");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3267");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3325");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3357");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3359");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3377");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3403");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3404");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3406");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3412");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3413");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3418");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3427");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3433");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3434");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3435");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3455");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3474");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3500");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3540");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3556");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3557");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3571");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3578");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3587");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3619");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3648");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3650");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3653");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3654");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3655");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3688");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3692");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3696");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3718");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 3733");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3757");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3759");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3767");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3792");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3818");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3822");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3830");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 3 WHERE `id` = 3834");
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 2 WHERE `id` = 3860");
			
			migrationBuilder.Sql(@"UPDATE `comic` SET `ImageType` = 1 WHERE `ImageType` = 0");
		}

		protected override void Down(MigrationBuilder migrationBuilder)
		{
			migrationBuilder.DropColumn(
				name: "ImageType",
				table: "comic");
		}
	}
}
