using QCExtensions.Application.Interfaces;

namespace QCExtensions.Application.Comics.Commands.SetTagline
{
	public class SetTaglineCommand : RequestWithToken
	{
		public override Permission RequiredPermissions => Permission.CanChangeComicData;

		public int ComicId { get; set; }
		public string Tagline { get; set; }
	}
}
