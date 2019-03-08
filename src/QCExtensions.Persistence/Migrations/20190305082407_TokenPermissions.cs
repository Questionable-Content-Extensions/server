using Microsoft.EntityFrameworkCore.Migrations;

namespace QCExtensions.Persistence.Migrations
{
    public partial class TokenPermissions : Migration
    {
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<bool>(
                name: "CanAddImageToItem",
                table: "token",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<bool>(
                name: "CanAddItemToComic",
                table: "token",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<bool>(
                name: "CanChangeComicData",
                table: "token",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<bool>(
                name: "CanChangeItemData",
                table: "token",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<bool>(
                name: "CanRemoveImageFromItem",
                table: "token",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<bool>(
                name: "CanRemoveItemFromComic",
                table: "token",
                nullable: false,
                defaultValue: false);

			migrationBuilder.Sql(@"UPDATE `token` SET `CanAddImageToItem` = 0");
			migrationBuilder.Sql(@"UPDATE `token` SET `CanAddItemToComic` = 1");
			migrationBuilder.Sql(@"UPDATE `token` SET `CanChangeComicData` = 1");
			migrationBuilder.Sql(@"UPDATE `token` SET `CanChangeItemData` = 1");
			migrationBuilder.Sql(@"UPDATE `token` SET `CanRemoveImageFromItem` = 1");
			migrationBuilder.Sql(@"UPDATE `token` SET `CanRemoveItemFromComic` = 1");
        }

        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "CanAddImageToItem",
                table: "token");

            migrationBuilder.DropColumn(
                name: "CanAddItemToComic",
                table: "token");

            migrationBuilder.DropColumn(
                name: "CanChangeComicData",
                table: "token");

            migrationBuilder.DropColumn(
                name: "CanChangeItemData",
                table: "token");

            migrationBuilder.DropColumn(
                name: "CanRemoveImageFromItem",
                table: "token");

            migrationBuilder.DropColumn(
                name: "CanRemoveItemFromComic",
                table: "token");
        }
    }
}
