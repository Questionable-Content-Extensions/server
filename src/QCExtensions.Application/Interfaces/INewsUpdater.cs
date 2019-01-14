using System.Collections.Generic;

namespace QCExtensions.Application.Interfaces
{
	public interface INewsUpdater
	{
		void CheckFor(int comic);

		ICollection<int> GetPendingUpdateEntries();
		void RemovePendingUpdateEntries(ICollection<int> updateEntries);
	}
}
