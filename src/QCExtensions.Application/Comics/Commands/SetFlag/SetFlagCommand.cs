using System;
using QCExtensions.Application.Interfaces;

namespace QCExtensions.Application.Comics.Commands.SetFlag
{
	public class SetFlagCommand : RequestWithToken
	{
		public enum FlagType
		{
			Invalid,
			IsGuestComic,
			IsNonCanon,
			HasNoCast,
			HasNoLocation,
			HasNoStoryline,
			HasNoTitle,
			HasNoTagline
		}

		public override Permission RequiredPermissions => Permission.CanChangeComicData;
		
		public int ComicId { get; set; }
		public FlagType Flag { get; set; }
		public bool FlagValue { get; set; }
	}
}
