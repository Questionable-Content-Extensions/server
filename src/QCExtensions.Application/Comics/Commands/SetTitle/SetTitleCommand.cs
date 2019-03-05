using QCExtensions.Application.Interfaces;

namespace QCExtensions.Application.Comics.Commands.SetTitle
{
	public class SetTitleCommand : RequestWithToken
	{
		public override Permission RequiredPermissions => Permission.CanChangeComicData;

		public int ComicId { get; set; }
		public string Title { get; set; }
	}
}
