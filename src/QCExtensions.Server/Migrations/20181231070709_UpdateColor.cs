using Microsoft.EntityFrameworkCore.Migrations;

namespace QCExtensions.Server.Migrations
{
	public partial class UpdateColor : Migration
	{
		protected override void Up(MigrationBuilder migrationBuilder)
		{
			migrationBuilder.AddColumn<byte>(
				name: "Color_Blue",
				table: "items",
				nullable: false,
				defaultValue: (byte)0);

			migrationBuilder.AddColumn<byte>(
				name: "Color_Green",
				table: "items",
				nullable: false,
				defaultValue: (byte)0);

			migrationBuilder.AddColumn<byte>(
				name: "Color_Red",
				table: "items",
				nullable: false,
				defaultValue: (byte)0);

			migrationBuilder.Sql(@"
				UPDATE `items` SET
					`Color_Red`= CONV(SUBSTRING(`color`, 1, 2), 16, 10),
					`Color_Green`= CONV(SUBSTRING(`color`, 3, 2), 16, 10),
					`Color_Blue`= CONV(SUBSTRING(`color`, 5, 2), 16, 10)
			");

			migrationBuilder.DropColumn(
				name: "color",
				table: "items");
		}

		protected override void Down(MigrationBuilder migrationBuilder)
		{
			migrationBuilder.AddColumn<string>(
				name: "color",
				table: "items",
				maxLength: 6,
				nullable: false,
				defaultValue: "");

			migrationBuilder.Sql(@"
				UPDATE `items` SET `color`= CONCAT(LPAD(HEX(`Color_Red`), 2, '0'), LPAD(HEX(`Color_Green`), 2, '0'), LPAD(HEX(`Color_Blue`), 2, '0'))
			");

			migrationBuilder.DropColumn(
				name: "Color_Blue",
				table: "items");

			migrationBuilder.DropColumn(
				name: "Color_Green",
				table: "items");

			migrationBuilder.DropColumn(
				name: "Color_Red",
				table: "items");
		}
	}
}
